package drivers

import (
	"database/sql"
	"fmt"
	"strings"

	_ "modernc.org/sqlite"
)

type SqliteDriver struct {
	db *sql.DB
}

func (d *SqliteDriver) Connect(config Config) error {
	if config.Database == "" {
		return fmt.Errorf("SQLite requires a database file path")
	}

	db, err := sql.Open("sqlite", config.Database)
	if err != nil {
		return err
	}

	if err := db.Ping(); err != nil {
		return err
	}

	d.db = db
	return nil
}

func (d *SqliteDriver) Disconnect() error {
	if d.db != nil {
		return d.db.Close()
	}
	return nil
}

func (d *SqliteDriver) GetSchema() (*SchemaResult, error) {
	tablesRows, err := d.db.Query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
	if err != nil {
		return nil, err
	}
	defer tablesRows.Close()

	var tables []SchemaObject = []SchemaObject{}
	for tablesRows.Next() {
		var name string
		if err := tablesRows.Scan(&name); err != nil {
			return nil, err
		}
		tables = append(tables, SchemaObject{Name: name, Schema: "main"})
	}

	viewsRows, err := d.db.Query("SELECT name FROM sqlite_master WHERE type='view'")
	if err != nil {
		return nil, err
	}
	defer viewsRows.Close()

	var views []SchemaObject = []SchemaObject{}
	for viewsRows.Next() {
		var name string
		if err := viewsRows.Scan(&name); err != nil {
			return nil, err
		}
		views = append(views, SchemaObject{Name: name, Schema: "main"})
	}

	return &SchemaResult{
		Tables:    tables,
		Views:     views,
		Functions: []SchemaObject{},
	}, nil
}

func (d *SqliteDriver) FetchTableData(tableName string, limit, offset int, sortColumn, sortDirection string) (*TableDataResult, error) {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))

	// Get PKs and column info
	colsRows, err := d.db.Query(fmt.Sprintf("PRAGMA table_info(%s)", safeTable))
	if err != nil {
		return nil, err
	}
	defer colsRows.Close()

	pkMap := make(map[string]bool)
	for colsRows.Next() {
		var cid int
		var name, dtype string
		var notnull, pk int
		var dflt interface{}
		if err := colsRows.Scan(&cid, &name, &dtype, &notnull, &dflt, &pk); err != nil {
			return nil, err
		}
		if pk == 1 {
			pkMap[name] = true
		}
	}

	orderClause := ""
	if sortColumn != "" {
		dir := "ASC"
		if strings.ToUpper(sortDirection) == "DESC" {
			dir = "DESC"
		}
		orderClause = fmt.Sprintf(" ORDER BY \"%s\" %s", strings.ReplaceAll(sortColumn, "\"", "\"\""), dir)
	}

	query := fmt.Sprintf("SELECT * FROM %s%s LIMIT ? OFFSET ?", safeTable, orderClause)
	rows, err := d.db.Query(query, limit, offset)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	data, fields, err := ScanRows(rows)
	if err != nil {
		return nil, err
	}

	for i := range fields {
		if pkMap[fields[i].Name] {
			fields[i].IsPrimaryKey = true
		}
	}

	var total int
	if err := d.db.QueryRow(fmt.Sprintf("SELECT COUNT(*) FROM %s", safeTable)).Scan(&total); err != nil {
		return nil, err
	}

	return &TableDataResult{
		Rows:       data,
		Fields:     fields,
		TotalCount: total,
	}, nil
}

func (d *SqliteDriver) Query(sql string) (*QueryResult, error) {
	rows, err := d.db.Query(sql)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	data, fields, err := ScanRows(rows)
	if err != nil {
		return nil, err
	}

	return &QueryResult{
		Rows:     data,
		Fields:   fields,
		RowCount: len(data),
	}, nil
}

func (d *SqliteDriver) UpdateCell(tableName, pkColumn string, pkValue interface{}, targetColumn string, newValue interface{}) error {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	safePk := fmt.Sprintf("\"%s\"", strings.ReplaceAll(pkColumn, "\"", "\"\""))
	safeTarget := fmt.Sprintf("\"%s\"", strings.ReplaceAll(targetColumn, "\"", "\"\""))

	query := fmt.Sprintf("UPDATE %s SET %s = ? WHERE %s = ?", safeTable, safeTarget, safePk)
	_, err := d.db.Exec(query, newValue, pkValue)
	return err
}

func (d *SqliteDriver) InsertRow(tableName string, data map[string]interface{}) (map[string]interface{}, error) {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	
	var res sql.Result
	var err error
	if len(data) == 0 {
		res, err = d.db.Exec(fmt.Sprintf("INSERT INTO %s DEFAULT VALUES", safeTable))
	} else {
		cols := make([]string, 0, len(data))
		values := make([]interface{}, 0, len(data))
		placeholders := make([]string, 0, len(data))
		for k, v := range data {
			cols = append(cols, fmt.Sprintf("\"%s\"", strings.ReplaceAll(k, "\"", "\"\"")))
			values = append(values, v)
			placeholders = append(placeholders, "?")
		}
		query := fmt.Sprintf("INSERT INTO %s (%s) VALUES (%s)", 
			safeTable, strings.Join(cols, ", "), strings.Join(placeholders, ", "))
		res, err = d.db.Exec(query, values...)
	}

	if err != nil {
		return nil, err
	}

	lastID, _ := res.LastInsertId()
	
	query := fmt.Sprintf("SELECT * FROM %s WHERE rowid = ?", safeTable)
	rows, err := d.db.Query(query, lastID)
	if err != nil {
		return map[string]interface{}{"lastID": lastID}, nil
	}
	defer rows.Close()
	rowRes, _, _ := ScanRows(rows)
	if len(rowRes) > 0 {
		return rowRes[0], nil
	}
	return map[string]interface{}{"lastID": lastID}, nil
}

func (d *SqliteDriver) DeleteRows(tableName, pkColumn string, pkValues []interface{}) error {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	safePk := fmt.Sprintf("\"%s\"", strings.ReplaceAll(pkColumn, "\"", "\"\""))
	
	placeholders := make([]string, len(pkValues))
	for i := range pkValues {
		placeholders[i] = "?"
	}

	query := fmt.Sprintf("DELETE FROM %s WHERE %s IN (%s)", 
		safeTable, safePk, strings.Join(placeholders, ", "))
	
	_, err := d.db.Exec(query, pkValues...)
	return err
}

func (d *SqliteDriver) GetTableColumns(tableName string) ([]TableColumn, error) {
	rows, err := d.db.Query(fmt.Sprintf("PRAGMA table_info(\"%s\")", strings.ReplaceAll(tableName, "\"", "\"\"")))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var cols []TableColumn
	for rows.Next() {
		var cid int
		var name, dtype string
		var notnull, pk int
		var dflt interface{}
		if err := rows.Scan(&cid, &name, &dtype, &notnull, &dflt, &pk); err != nil {
			return nil, err
		}
		cols = append(cols, TableColumn{
			Name:     name,
			DataType: dtype,
			Nullable: notnull == 0,
			Default:  dflt,
		})
	}
	return cols, nil
}

func (d *SqliteDriver) AlterTable(tableName string, operations []AlterOperation) error {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	
	for _, op := range operations {
		var query string
		switch op.Type {
		case "ADD_COLUMN":
			safeCol := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.Name, "\"", "\"\""))
			query = fmt.Sprintf("ALTER TABLE %s ADD COLUMN %s %s", safeTable, safeCol, op.DataType)
			if op.Nullable != nil && !*op.Nullable {
				query += " NOT NULL"
			}
			if op.Default != nil {
				query += fmt.Sprintf(" DEFAULT %v", op.Default)
			}
		case "DROP_COLUMN":
			safeCol := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.Name, "\"", "\"\""))
			query = fmt.Sprintf("ALTER TABLE %s DROP COLUMN %s", safeTable, safeCol)
		case "RENAME_COLUMN":
			safeOld := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.OldName, "\"", "\"\""))
			safeNew := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.NewName, "\"", "\"\""))
			query = fmt.Sprintf("ALTER TABLE %s RENAME COLUMN %s TO %s", safeTable, safeOld, safeNew)
		}
		
		if query != "" {
			if _, err := d.db.Exec(query); err != nil {
				return err
			}
		}
	}
	return nil
}

func (d *SqliteDriver) ExportToCSV(tableName, exportPath string) error {
	return ExportToCSVHelper(d.db, "\""+strings.ReplaceAll(tableName, "\"", "\"\"")+"\"", exportPath, "\"")
}
