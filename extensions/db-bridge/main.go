package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/vanto/table-view/db-bridge/internal/bridge"
	"github.com/vanto/table-view/db-bridge/internal/drivers"
	"github.com/vanto/table-view/db-bridge/internal/pool"
)

func main() {
	// Try to get the executable path to put the log file next to it
	exePath, err := os.Executable()
	var logFile string
	if err == nil {
		logFile = strings.TrimSuffix(exePath, ".exe") + ".log"
	} else {
		logFile = "db-bridge.log"
	}

	f, err := os.OpenFile(logFile, os.O_RDWR|os.O_CREATE|os.O_APPEND, 0666)
	if err == nil {
		log.SetOutput(f)
		defer f.Close()
	}
	log.Println("--- db-bridge extension starting ---")
	log.Println("Working directory:", func() string { wd, _ := os.Getwd(); return wd }())
	log.Println("Executable path:", exePath)

	var auth bridge.AuthInfo

	// Use Decoder instead of ReadAll to avoid hanging if stdin doesn't close
	decoder := json.NewDecoder(os.Stdin)
	if err := decoder.Decode(&auth); err != nil {
		log.Println("Error decoding auth from stdin:", err)
	} else {
		log.Printf("Received auth from stdin: %+v\n", auth)
	}

	// Fallback/Override with flags
	port := flag.String("nl-port", auth.NLPort, "Neutralino port")
	token := flag.String("nl-token", auth.NLToken, "Neutralino token")
	extId := flag.String("nl-extension-id", auth.NLExtId, "Extension ID")
	connectToken := flag.String("nl-connect-token", auth.NLConnectToken, "Connect token")
	flag.Parse()

	if *port != "" {
		auth.NLPort = *port
	}
	if *token != "" {
		auth.NLToken = *token
	}
	if *extId != "" {
		auth.NLExtId = *extId
	}
	if *connectToken != "" {
		auth.NLConnectToken = *connectToken
	}

	if auth.NLPort == "" || auth.NLToken == "" || auth.NLExtId == "" {
		log.Fatal("Missing NeutralinoJS extension connection details.")
	}

	b := bridge.NewBridge(auth)
	if err := b.Connect(); err != nil {
		log.Fatal("Failed to connect to NeutralinoJS:", err)
	}
	defer b.Close()

	// ─── Connection Pool ─────────────────────────────────────────────────────
	connPool := pool.New()
	defer connPool.CloseAll()

	log.Printf("Extension %s connected to NeutralinoJS.\n", auth.NLExtId)

	b.Listen(func(msg bridge.Message) {
		if !strings.HasPrefix(msg.Event, "dbBridge.") {
			return
		}

		action := strings.TrimPrefix(msg.Event, "dbBridge.")

		var payload struct {
			ReqId         string                   `json:"reqId"`
			ConnectionId  string                   `json:"connectionId"`
			Config        drivers.Config           `json:"config"`
			TableName     string                   `json:"tableName"`
			Limit         int                      `json:"limit"`
			Offset        int                      `json:"offset"`
			SortColumn    string                   `json:"sortColumn"`
			SortDirection string                   `json:"sortDirection"`
			Filter        string                   `json:"filter"`
			SQL           string                   `json:"sql"`
			PKColumn      string                   `json:"pkColumn"`
			PKValue       interface{}              `json:"pkValue"`
			TargetColumn  string                   `json:"targetColumn"`
			NewValue      interface{}              `json:"newValue"`
			ExportPath    string                   `json:"exportPath"`
			Data          map[string]interface{}   `json:"data"`
			PKValues      []interface{}            `json:"pkValues"`
			Operations    []drivers.AlterOperation `json:"operations"`
			AllSchemas    bool                     `json:"allSchemas"`
		}

		if err := json.Unmarshal(msg.Data, &payload); err != nil {
			log.Println("Error unmarshaling payload:", err)
			return
		}

		go func() {
			switch action {

			// ── testConnection: doesn't touch the pool (ephemeral) ────────────
			case "testConnection":
				d := getDriver(payload.Config.Type)
				if d == nil {
					b.Broadcast("dbBridge.testConnectionResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": false, "error": "unsupported driver type",
					})
					return
				}
				err := d.Connect(payload.Config)
				if err == nil {
					d.Disconnect()
					b.Broadcast("dbBridge.testConnectionResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": true,
					})
				} else {
					b.Broadcast("dbBridge.testConnectionResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": false, "error": err.Error(),
					})
				}

			// ── connect: open a new connection and register in pool ───────────
			case "connect":
				d := getDriver(payload.Config.Type)
				if d == nil {
					b.Broadcast("dbBridge.connectResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": false, "error": "unsupported driver type",
					})
					return
				}
				connId := payload.ConnectionId
				if connId == "" {
					connId = payload.Config.Type + "-default"
				}
				err := d.Connect(payload.Config)
				if err == nil {
					connPool.Put(connId, d)
					log.Printf("pool: registered connection %s", connId)
					b.Broadcast("dbBridge.connectResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": true,
					})
				} else {
					b.Broadcast("dbBridge.connectResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": false, "error": err.Error(),
					})
				}

			// ── all other actions: route via pool ─────────────────────────────
			default:
				driver := connPool.Get(payload.ConnectionId)
				if driver == nil {
					log.Printf("pool: no driver for connectionId=%q (action=%s)", payload.ConnectionId, action)
					handleResult(b, "dbBridge."+action+"Result", payload.ReqId, nil,
						fmt.Errorf("not connected (connectionId=%q)", payload.ConnectionId))
					return
				}

				switch action {
				case "getSchema":
					res, err := driver.GetSchema(payload.AllSchemas)
					handleResult(b, "dbBridge.getSchemaResult", payload.ReqId, res, err)

				case "fetchTableData":
					res, err := driver.FetchTableData(payload.TableName, payload.Limit, payload.Offset, payload.SortColumn, payload.SortDirection)
					handleResult(b, "dbBridge.fetchTableDataResult", payload.ReqId, res, err)

				case "executeQuery":
					res, err := driver.Query(payload.SQL)
					handleResult(b, "dbBridge.executeQueryResult", payload.ReqId, res, err)

				case "updateCell":
					err := driver.UpdateCell(payload.TableName, payload.PKColumn, payload.PKValue, payload.TargetColumn, payload.NewValue)
					handleResult(b, "dbBridge.updateCellResult", payload.ReqId, nil, err)

				case "exportCSV":
					err := driver.ExportToCSV(payload.TableName, payload.ExportPath)
					handleResult(b, "dbBridge.exportCSVResult", payload.ReqId, nil, err)

				case "insertRow":
					res, err := driver.InsertRow(payload.TableName, payload.Data)
					if err == nil {
						b.Broadcast("dbBridge.insertRowResult", map[string]interface{}{
							"reqId": payload.ReqId, "success": true, "row": res,
						})
					} else {
						handleResult(b, "dbBridge.insertRowResult", payload.ReqId, nil, err)
					}

				case "deleteRows":
					err := driver.DeleteRows(payload.TableName, payload.PKColumn, payload.PKValues)
					handleResult(b, "dbBridge.deleteRowsResult", payload.ReqId, nil, err)

				case "getTableColumns":
					res, err := driver.GetTableColumns(payload.TableName)
					if err == nil {
						b.Broadcast("dbBridge.getTableColumnsResult", map[string]interface{}{
							"reqId": payload.ReqId, "success": true, "columns": res,
						})
					} else {
						handleResult(b, "dbBridge.getTableColumnsResult", payload.ReqId, nil, err)
					}

				case "alterTable":
					err := driver.AlterTable(payload.TableName, payload.Operations)
					handleResult(b, "dbBridge.alterTableResult", payload.ReqId, nil, err)

				default:
					log.Printf("Unknown action: %s", action)
				}
			}
		}()
	})
}

func getDriver(driverType string) drivers.DatabaseDriver {
	if driverType == "postgres" || driverType == "postgresql" {
		return &drivers.PostgresDriver{}
	}
	if driverType == "mysql" {
		return &drivers.MySqlDriver{}
	}
	if driverType == "sqlite" {
		return &drivers.SqliteDriver{}
	}
	return nil
}

func handleResult(b *bridge.Bridge, event, reqId string, res interface{}, err error) {
	resp := map[string]interface{}{
		"reqId":   reqId,
		"success": err == nil,
	}
	if err != nil {
		resp["error"] = err.Error()
	} else if res != nil {
		if strings.HasPrefix(event, "dbBridge.get") {
			key := strings.TrimPrefix(event, "dbBridge.get")
			key = strings.TrimSuffix(key, "Result")
			key = strings.ToLower(key[:1]) + key[1:]
			resp[key] = res
		} else {
			data, _ := json.Marshal(res)
			json.Unmarshal(data, &resp)
		}
	}
	b.Broadcast(event, resp)
}
