package drivers

import (
	"database/sql"
	"fmt"
	"strings"

	_ "github.com/go-sql-driver/mysql"
)

type MySqlDriver struct {
	db *sql.DB
}

func (d *MySqlDriver) Connect(config Config) error {
	dsn := fmt.Sprintf("%s:%s@tcp(%s:%d)/%s?parseTime=true",
		config.Username, config.Password, config.Host, config.Port, config.Database)

	db, err := sql.Open("mysql", dsn)
	if err != nil {
		return err
	}

	if err := db.Ping(); err != nil {
		return err
	}

	d.db = db
	return nil
}

func (d *MySqlDriver) Disconnect() error {
	if d.db != nil {
		return d.db.Close()
	}
	return nil
}

func (d *MySqlDriver) GetSchema(allSchemas bool) (*SchemaResult, error) {
	whereClause := "WHERE TABLE_SCHEMA = DATABASE()"
	if allSchemas {
		whereClause = "WHERE TABLE_SCHEMA NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys')"
	}

	tablesRows, err := d.db.Query(fmt.Sprintf(`
		SELECT TABLE_NAME as name, TABLE_SCHEMA as schema_name, 'table' as type
		FROM information_schema.TABLES
		%s AND TABLE_TYPE = 'BASE TABLE'
	`, whereClause))
	if err != nil {
		return nil, err
	}
	defer tablesRows.Close()

	var tables []SchemaObject = []SchemaObject{}
	for tablesRows.Next() {
		var name, schemaName, ttype string
		if err := tablesRows.Scan(&name, &schemaName, &ttype); err != nil {
			return nil, err
		}
		tables = append(tables, SchemaObject{Name: name, Schema: schemaName})
	}

	viewsRows, err := d.db.Query(fmt.Sprintf(`
		SELECT TABLE_NAME as name, TABLE_SCHEMA as schema_name, 'view' as type
		FROM information_schema.TABLES
		%s AND TABLE_TYPE = 'VIEW'
	`, whereClause))
	if err != nil {
		return nil, err
	}
	defer viewsRows.Close()

	var views []SchemaObject = []SchemaObject{}
	for viewsRows.Next() {
		var name, schemaName, ttype string
		if err := viewsRows.Scan(&name, &schemaName, &ttype); err != nil {
			return nil, err
		}
		views = append(views, SchemaObject{Name: name, Schema: schemaName})
	}

	routinesRows, err := d.db.Query(fmt.Sprintf(`
		SELECT ROUTINE_NAME as name, ROUTINE_SCHEMA as schema_name, ROUTINE_TYPE as type
		FROM information_schema.ROUTINES
		%s
	`, strings.Replace(whereClause, "TABLE_SCHEMA", "ROUTINE_SCHEMA", 1)))
	if err != nil {
		return nil, err
	}
	defer routinesRows.Close()

	var funcs []SchemaObject = []SchemaObject{}
	for routinesRows.Next() {
		var name, schemaName, rtype string
		if err := routinesRows.Scan(&name, &schemaName, &rtype); err != nil {
			return nil, err
		}
		funcs = append(funcs, SchemaObject{Name: name, Schema: schemaName, Type: rtype})
	}

	schemasRows, err := d.db.Query(fmt.Sprintf(`
		SELECT SCHEMA_NAME as name
		FROM information_schema.SCHEMATA
		%s
	`, strings.Replace(whereClause, "TABLE_SCHEMA", "SCHEMA_NAME", 1)))
	if err != nil {
		return nil, err
	}
	defer schemasRows.Close()

	var schemas []SchemaObject = []SchemaObject{}
	for schemasRows.Next() {
		var name string
		if err := schemasRows.Scan(&name); err != nil {
			return nil, err
		}
		schemas = append(schemas, SchemaObject{Name: name})
	}

	return &SchemaResult{
		Tables:    tables,
		Views:     views,
		Functions: funcs,
		Schemas:   schemas,
	}, nil
}

func (d *MySqlDriver) FetchTableData(tableName string, limit, offset int, sortColumn, sortDirection string) (*TableDataResult, error) {
	safeTable := fmt.Sprintf("`%s`Status", strings.ReplaceAll(tableName, "`", "``"))
	safeTable = fmt.Sprintf("`%s`", strings.ReplaceAll(tableName, "`", "``"))

	// Get PKs
	pkRows, err := d.db.Query(`
		SELECT COLUMN_NAME
		FROM information_schema.COLUMNS
		WHERE TABLE_SCHEMA = DATABASE()
			AND TABLE_NAME = ?
			AND COLUMN_KEY = 'PRI'
	`, tableName)
	if err != nil {
		return nil, err
	}
	defer pkRows.Close()

	pkMap := make(map[string]bool)
	for pkRows.Next() {
		var col string
		if err := pkRows.Scan(&col); err != nil {
			return nil, err
		}
		pkMap[col] = true
	}

	orderClause := ""
	if sortColumn != "" {
		dir := "ASC"
		if strings.ToUpper(sortDirection) == "DESC" {
			dir = "DESC"
		}
		orderClause = fmt.Sprintf(" ORDER BY `%s` %s", strings.ReplaceAll(sortColumn, "`", "``"), dir)
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

func (d *MySqlDriver) Query(sql string) (*QueryResult, error) {
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

func (d *MySqlDriver) UpdateCell(tableName, pkColumn string, pkValue interface{}, targetColumn string, newValue interface{}) error {
	safeTable := fmt.Sprintf("`%s`", strings.ReplaceAll(tableName, "`", "``"))
	safePk := fmt.Sprintf("`%s`", strings.ReplaceAll(pkColumn, "`", "``"))
	safeTarget := fmt.Sprintf("`%s`", strings.ReplaceAll(targetColumn, "`", "``"))

	query := fmt.Sprintf("UPDATE %s SET %s = ? WHERE %s = ?", safeTable, safeTarget, safePk)
	_, err := d.db.Exec(query, newValue, pkValue)
	return err
}

func (d *MySqlDriver) InsertRow(tableName string, data map[string]interface{}) (map[string]interface{}, error) {
	safeTable := fmt.Sprintf("`%s`", strings.ReplaceAll(tableName, "`", "``"))
	
	var res sql.Result
	var err error
	if len(data) == 0 {
		res, err = d.db.Exec(fmt.Sprintf("INSERT INTO %s () VALUES ()", safeTable))
	} else {
		cols := make([]string, 0, len(data))
		values := make([]interface{}, 0, len(data))
		placeholders := make([]string, 0, len(data))
		for k, v := range data {
			cols = append(cols, fmt.Sprintf("`%s`Status", strings.ReplaceAll(k, "`", "``"))) // wait fixed below
			cols[len(cols)-1] = fmt.Sprintf("`%s`", strings.ReplaceAll(k, "`", "``"))
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
	
	// MySQL usually needs another query to get the full row if it has defaults
	// But we'll try to guess based on ID for now, or just return ID
	query := fmt.Sprintf("SELECT * FROM %s WHERE id = ?", safeTable)
	rows, err := d.db.Query(query, lastID)
	if err != nil {
		return map[string]interface{}{"insertId": lastID}, nil
	}
	defer rows.Close()
	rowRes, _, _ := ScanRows(rows)
	if len(rowRes) > 0 {
		return rowRes[0], nil
	}
	return map[string]interface{}{"insertId": lastID}, nil
}

func (d *MySqlDriver) DeleteRows(tableName, pkColumn string, pkValues []interface{}) error {
	safeTable := fmt.Sprintf("`%s`", strings.ReplaceAll(tableName, "`", "``"))
	safePk := fmt.Sprintf("`%s`", strings.ReplaceAll(pkColumn, "`", "``"))
	
	placeholders := make([]string, len(pkValues))
	for i := range pkValues {
		placeholders[i] = "?"
	}

	query := fmt.Sprintf("DELETE FROM %s WHERE %s IN (%s)", 
		safeTable, safePk, strings.Join(placeholders, ", "))
	
	_, err := d.db.Exec(query, pkValues...)
	return err
}

func (d *MySqlDriver) GetTableColumns(tableName string) ([]TableColumn, error) {
	rows, err := d.db.Query(`
		SELECT 
			COLUMN_NAME as name, 
			DATA_TYPE as dataType, 
			IS_NULLABLE as isNullable, 
			COLUMN_DEFAULT as columnDefault 
		FROM INFORMATION_SCHEMA.COLUMNS 
		WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
		ORDER BY ORDINAL_POSITION
	`, tableName)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var cols []TableColumn
	for rows.Next() {
		var c TableColumn
		var nullable string
		if err := rows.Scan(&c.Name, &c.DataType, &nullable, &c.Default); err != nil {
			return nil, err
		}
		c.Nullable = nullable == "YES"
		cols = append(cols, c)
	}
	return cols, nil
}

func (d *MySqlDriver) AlterTable(tableName string, operations []AlterOperation) error {
	safeTable := fmt.Sprintf("`%s`", strings.ReplaceAll(tableName, "`", "``"))
	
	for _, op := range operations {
		var query string
		switch op.Type {
		case "ADD_COLUMN":
			safeCol := fmt.Sprintf("`%s`", strings.ReplaceAll(op.Name, "`", "``"))
			query = fmt.Sprintf("ALTER TABLE %s ADD COLUMN %s %s", safeTable, safeCol, op.DataType)
			if op.Nullable != nil && !*op.Nullable {
				query += " NOT NULL"
			}
			if op.Default != nil {
				query += fmt.Sprintf(" DEFAULT %v", op.Default)
			}
		case "DROP_COLUMN":
			safeCol := fmt.Sprintf("`%s`", strings.ReplaceAll(op.Name, "`", "``"))
			query = fmt.Sprintf("ALTER TABLE %s DROP COLUMN %s", safeTable, safeCol)
		case "RENAME_COLUMN":
			safeOld := fmt.Sprintf("`%s`", strings.ReplaceAll(op.OldName, "`", "``"))
			safeNew := fmt.Sprintf("`%s`", strings.ReplaceAll(op.NewName, "`", "``"))
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

func (d *MySqlDriver) ExportToCSV(tableName, exportPath string) error {
	return ExportToCSVHelper(d.db, "`"+strings.ReplaceAll(tableName, "`", "``")+"`", exportPath, "`")
}
