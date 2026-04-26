package drivers

import (
	"database/sql"
	"fmt"
	"strings"

	_ "github.com/jackc/pgx/v5/stdlib"
)

type PostgresDriver struct {
	db *sql.DB
}

func (d *PostgresDriver) Connect(config Config) error {
	sslMode := "disable"
	if config.SSL {
		sslMode = "require"
	}

	dsn := fmt.Sprintf("postgres://%s:%s@%s:%d/%s?sslmode=%s&default_query_exec_mode=simple_protocol",
		config.Username, config.Password, config.Host, config.Port, config.Database, sslMode)

	db, err := sql.Open("pgx", dsn)
	if err != nil {
		return err
	}

	if err := db.Ping(); err != nil {
		return err
	}

	d.db = db
	return nil
}

func (d *PostgresDriver) Disconnect() error {
	if d.db != nil {
		return d.db.Close()
	}
	return nil
}

func (d *PostgresDriver) GetSchema() (*SchemaResult, error) {
	tablesRows, err := d.db.Query(`
		SELECT table_name 
		FROM information_schema.tables 
		WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
		ORDER BY table_name;
	`)
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
		tables = append(tables, SchemaObject{Name: name})
	}

	viewsRows, err := d.db.Query(`
		SELECT table_name 
		FROM information_schema.views 
		WHERE table_schema = 'public'
		ORDER BY table_name;
	`)
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
		views = append(views, SchemaObject{Name: name})
	}

	funcsRows, err := d.db.Query(`
		SELECT routine_name 
		FROM information_schema.routines 
		WHERE routine_schema = 'public' AND routine_type = 'FUNCTION'
		ORDER BY routine_name;
	`)
	if err != nil {
		return nil, err
	}
	defer funcsRows.Close()

	var funcs []SchemaObject = []SchemaObject{}
	for funcsRows.Next() {
		var name string
		if err := funcsRows.Scan(&name); err != nil {
			return nil, err
		}
		funcs = append(funcs, SchemaObject{Name: name})
	}

	return &SchemaResult{
		Tables:    tables,
		Views:     views,
		Functions: funcs,
		Schemas:   []SchemaObject{{Name: "public"}},
	}, nil
}

func (d *PostgresDriver) FetchTableData(tableName string, limit, offset int, sortColumn, sortDirection string) (*TableDataResult, error) {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	
	// Get PKs
	pkRows, err := d.db.Query(`
		SELECT kcu.column_name
		FROM information_schema.table_constraints tco
		JOIN information_schema.key_column_usage kcu 
			ON kcu.constraint_name = tco.constraint_name
			AND kcu.constraint_schema = tco.constraint_schema
		WHERE tco.constraint_type = 'PRIMARY KEY' 
			AND kcu.table_name = $1
			AND tco.table_schema = 'public';
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
		orderClause = fmt.Sprintf(" ORDER BY \"%s\" %s", strings.ReplaceAll(sortColumn, "\"", "\"\""), dir)
	}

	query := fmt.Sprintf("SELECT * FROM public.%s%s LIMIT $1 OFFSET $2", safeTable, orderClause)
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

	countQuery := fmt.Sprintf("SELECT COUNT(*) FROM public.%s", safeTable)
	var total int
	if err := d.db.QueryRow(countQuery).Scan(&total); err != nil {
		return nil, err
	}

	return &TableDataResult{
		Rows:       data,
		Fields:     fields,
		TotalCount: total,
	}, nil
}

func (d *PostgresDriver) Query(sql string) (*QueryResult, error) {
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

func (d *PostgresDriver) UpdateCell(tableName, pkColumn string, pkValue interface{}, targetColumn string, newValue interface{}) error {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	safePk := fmt.Sprintf("\"%s\"", strings.ReplaceAll(pkColumn, "\"", "\"\""))
	safeTarget := fmt.Sprintf("\"%s\"", strings.ReplaceAll(targetColumn, "\"", "\"\""))

	query := fmt.Sprintf("UPDATE public.%s SET %s = $1 WHERE %s = $2", safeTable, safeTarget, safePk)
	_, err := d.db.Exec(query, newValue, pkValue)
	return err
}

func (d *PostgresDriver) InsertRow(tableName string, data map[string]interface{}) (map[string]interface{}, error) {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	
	if len(data) == 0 {
		query := fmt.Sprintf("INSERT INTO public.%s DEFAULT VALUES RETURNING *", safeTable)
		rows, err := d.db.Query(query)
		if err != nil {
			return nil, err
		}
		defer rows.Close()
		res, _, err := ScanRows(rows)
		if err != nil || len(res) == 0 {
			return nil, err
		}
		return res[0], nil
	}

	cols := make([]string, 0, len(data))
	values := make([]interface{}, 0, len(data))
	placeholders := make([]string, 0, len(data))
	
	i := 1
	for k, v := range data {
		cols = append(cols, fmt.Sprintf("\"%s\"", strings.ReplaceAll(k, "\"", "\"\"")))
		values = append(values, v)
		placeholders = append(placeholders, fmt.Sprintf("$%d", i))
		i++
	}

	query := fmt.Sprintf("INSERT INTO public.%s (%s) VALUES (%s) RETURNING *", 
		safeTable, strings.Join(cols, ", "), strings.Join(placeholders, ", "))
	
	rows, err := d.db.Query(query, values...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res, _, err := ScanRows(rows)
	if err != nil || len(res) == 0 {
		return nil, err
	}
	return res[0], nil
}

func (d *PostgresDriver) DeleteRows(tableName, pkColumn string, pkValues []interface{}) error {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	safePk := fmt.Sprintf("\"%s\"", strings.ReplaceAll(pkColumn, "\"", "\"\""))
	
	placeholders := make([]string, len(pkValues))
	for i := range pkValues {
		placeholders[i] = fmt.Sprintf("$%d", i+1)
	}

	query := fmt.Sprintf("DELETE FROM public.%s WHERE %s IN (%s)", 
		safeTable, safePk, strings.Join(placeholders, ", "))
	
	_, err := d.db.Exec(query, pkValues...)
	return err
}

func (d *PostgresDriver) GetTableColumns(tableName string) ([]TableColumn, error) {
	rows, err := d.db.Query(`
		SELECT 
			column_name, data_type, is_nullable, column_default
		FROM information_schema.columns 
		WHERE table_schema = 'public' AND table_name = $1
		ORDER BY ordinal_position;
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

func (d *PostgresDriver) AlterTable(tableName string, operations []AlterOperation) error {
	safeTable := fmt.Sprintf("\"%s\"", strings.ReplaceAll(tableName, "\"", "\"\""))
	
	for _, op := range operations {
		var query string
		switch op.Type {
		case "ADD_COLUMN":
			safeCol := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.Name, "\"", "\"\""))
			query = fmt.Sprintf("ALTER TABLE public.%s ADD COLUMN %s %s", safeTable, safeCol, op.DataType)
			if op.Nullable != nil && !*op.Nullable {
				query += " NOT NULL"
			}
			if op.Default != nil {
				query += fmt.Sprintf(" DEFAULT %v", op.Default)
			}
		case "DROP_COLUMN":
			safeCol := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.Name, "\"", "\"\""))
			query = fmt.Sprintf("ALTER TABLE public.%s DROP COLUMN %s", safeTable, safeCol)
		case "RENAME_COLUMN":
			safeOld := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.OldName, "\"", "\"\""))
			safeNew := fmt.Sprintf("\"%s\"", strings.ReplaceAll(op.NewName, "\"", "\"\""))
			query = fmt.Sprintf("ALTER TABLE public.%s RENAME COLUMN %s TO %s", safeTable, safeOld, safeNew)
		}
		
		if query != "" {
			if _, err := d.db.Exec(query); err != nil {
				return err
			}
		}
	}
	return nil
}

func (d *PostgresDriver) ExportToCSV(tableName, exportPath string) error {
	return ExportToCSVHelper(d.db, "public.\""+strings.ReplaceAll(tableName, "\"", "\"\"")+"\"", exportPath, "\"")
}
