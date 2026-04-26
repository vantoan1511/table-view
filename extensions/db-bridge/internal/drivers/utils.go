package drivers

import (
	"database/sql"
	"encoding/csv"
	"fmt"
	"os"
)

// ScanRows converts sql.Rows to a slice of maps
func ScanRows(rows *sql.Rows) ([]map[string]interface{}, []ColumnInfo, error) {
	cols, err := rows.Columns()
	if err != nil {
		return nil, nil, err
	}

	types, _ := rows.ColumnTypes()
	fields := make([]ColumnInfo, len(cols))
	for i, col := range cols {
		dataTypeID := interface{}(0)
		if i < len(types) {
			dataTypeID = types[i].DatabaseTypeName()
		}
		fields[i] = ColumnInfo{
			Name:       col,
			DataTypeID: dataTypeID,
		}
	}

	var result []map[string]interface{}
	for rows.Next() {
		columns := make([]interface{}, len(cols))
		columnPointers := make([]interface{}, len(cols))
		for i := range columns {
			columnPointers[i] = &columns[i]
		}

		if err := rows.Scan(columnPointers...); err != nil {
			return nil, nil, err
		}

		m := make(map[string]interface{})
		for i, colName := range cols {
			val := columns[i]
			b, ok := val.([]byte)
			if ok {
				m[colName] = string(b)
			} else {
				m[colName] = val
			}
		}
		result = append(result, m)
	}

	return result, fields, nil
}

// ExportToCSVHelper provides a generic implementation for CSV export
func ExportToCSVHelper(db *sql.DB, tableName, exportPath, quoteChar string) error {
	file, err := os.Create(exportPath)
	if err != nil {
		return err
	}
	defer file.Close()

	writer := csv.NewWriter(file)
	defer writer.Flush()

	query := fmt.Sprintf("SELECT * FROM %s", tableName)
	rows, err := db.Query(query)
	if err != nil {
		return err
	}
	defer rows.Close()

	cols, err := rows.Columns()
	if err != nil {
		return err
	}

	if err := writer.Write(cols); err != nil {
		return err
	}

	values := make([]interface{}, len(cols))
	valuePtrs := make([]interface{}, len(cols))
	for i := range values {
		valuePtrs[i] = &values[i]
	}

	for rows.Next() {
		if err := rows.Scan(valuePtrs...); err != nil {
			return err
		}

		record := make([]string, len(cols))
		for i, val := range values {
			if val == nil {
				record[i] = ""
			} else {
				record[i] = fmt.Sprintf("%v", val)
			}
		}
		if err := writer.Write(record); err != nil {
			return err
		}
	}

	return nil
}
