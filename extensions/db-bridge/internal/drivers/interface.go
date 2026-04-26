package drivers

type Config struct {
	Type              string `json:"type"`
	Host              string `json:"host"`
	Port              int    `json:"port"`
	Username          string `json:"username"`
	Password          string `json:"password"`
	Database          string `json:"database"`
	SSL               bool   `json:"ssl"`
	ConnectionTimeout int    `json:"connectionTimeout"`
}

type ColumnInfo struct {
	Name         string      `json:"name"`
	DataTypeID   interface{} `json:"dataTypeID"`
	IsPrimaryKey bool        `json:"isPrimaryKey"`
}

type QueryResult struct {
	Rows      []map[string]interface{} `json:"rows"`
	Fields    []ColumnInfo             `json:"fields"`
	RowCount  int                      `json:"rowCount"`
}

type TableDataResult struct {
	Rows       []map[string]interface{} `json:"rows"`
	Fields     []ColumnInfo             `json:"fields"`
	TotalCount int                      `json:"totalCount"`
}

type SchemaObject struct {
	Name   string `json:"name"`
	Schema string `json:"schema,omitempty"`
	Type   string `json:"type,omitempty"`
}

type SchemaResult struct {
	Tables    []SchemaObject `json:"tables"`
	Views     []SchemaObject `json:"views"`
	Functions []SchemaObject `json:"functions"`
	Schemas   []SchemaObject `json:"schemas,omitempty"`
}

type TableColumn struct {
	Name     string      `json:"name"`
	DataType string      `json:"dataType"`
	Nullable bool        `json:"nullable"`
	Default  interface{} `json:"default"`
}

type AlterOperation struct {
	Type     string      `json:"type"`
	Name     string      `json:"name,omitempty"`
	OldName  string      `json:"oldName,omitempty"`
	NewName  string      `json:"newName,omitempty"`
	DataType string      `json:"dataType,omitempty"`
	Nullable *bool       `json:"nullable,omitempty"`
	Default  interface{} `json:"default,omitempty"`
}

type DatabaseDriver interface {
	Connect(config Config) error
	Disconnect() error
	GetSchema() (*SchemaResult, error)
	FetchTableData(tableName string, limit, offset int, sortColumn, sortDirection string) (*TableDataResult, error)
	Query(sql string) (*QueryResult, error)
	UpdateCell(tableName, pkColumn string, pkValue interface{}, targetColumn string, newValue interface{}) error
	InsertRow(tableName string, data map[string]interface{}) (map[string]interface{}, error)
	DeleteRows(tableName, pkColumn string, pkValues []interface{}) error
	GetTableColumns(tableName string) ([]TableColumn, error)
	AlterTable(tableName string, operations []AlterOperation) error
	ExportToCSV(tableName, exportPath string) error
}
