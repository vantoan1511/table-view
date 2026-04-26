package main

import (
	"encoding/json"
	"flag"
	"io"
	"log"
	"net/http"
	"os"
	"strings"

	"github.com/vanto/table-view/db-bridge/internal/bridge"
	"github.com/vanto/table-view/db-bridge/internal/drivers"
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

	var activeDriver drivers.DatabaseDriver

	log.Printf("Extension %s connected to NeutralinoJS.\n", auth.NLExtId)

	b.Listen(func(msg bridge.Message) {
		if !strings.HasPrefix(msg.Event, "dbBridge.") {
			return
		}

		action := strings.TrimPrefix(msg.Event, "dbBridge.")

		var payload struct {
			ReqId         string                   `json:"reqId"`
			Config        drivers.Config           `json:"config"`
			TableName     string                   `json:"tableName"`
			Limit         int                      `json:"limit"`
			Offset        int                      `json:"offset"`
			SortColumn    string                   `json:"sortColumn"`
			SortDirection string                   `json:"sortDirection"`
			SQL           string                   `json:"sql"`
			PKColumn      string                   `json:"pkColumn"`
			PKValue       interface{}              `json:"pkValue"`
			TargetColumn  string                   `json:"targetColumn"`
			NewValue      interface{}              `json:"newValue"`
			ExportPath    string                   `json:"exportPath"`
			Data          map[string]interface{}   `json:"data"`
			PKValues      []interface{}            `json:"pkValues"`
			Operations    []drivers.AlterOperation `json:"operations"`
			DownloadUrl   string                   `json:"downloadUrl"`
		}

		if err := json.Unmarshal(msg.Data, &payload); err != nil {
			log.Println("Error unmarshaling payload:", err)
			return
		}

		go func() {
			switch action {
			case "testConnection":
				d := getDriver(payload.Config.Type)
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

			case "connect":
				activeDriver = getDriver(payload.Config.Type)
				err := activeDriver.Connect(payload.Config)
				if err == nil {
					b.Broadcast("dbBridge.connectResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": true,
					})
				} else {
					b.Broadcast("dbBridge.connectResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": false, "error": err.Error(),
					})
				}

			case "getSchema":
				res, err := activeDriver.GetSchema()
				handleResult(b, "dbBridge.getSchemaResult", payload.ReqId, res, err)

			case "fetchTableData":
				res, err := activeDriver.FetchTableData(payload.TableName, payload.Limit, payload.Offset, payload.SortColumn, payload.SortDirection)
				handleResult(b, "dbBridge.fetchTableDataResult", payload.ReqId, res, err)

			case "executeQuery":
				res, err := activeDriver.Query(payload.SQL)
				handleResult(b, "dbBridge.executeQueryResult", payload.ReqId, res, err)

			case "updateCell":
				err := activeDriver.UpdateCell(payload.TableName, payload.PKColumn, payload.PKValue, payload.TargetColumn, payload.NewValue)
				handleResult(b, "dbBridge.updateCellResult", payload.ReqId, nil, err)

			case "exportCSV":
				err := activeDriver.ExportToCSV(payload.TableName, payload.ExportPath)
				handleResult(b, "dbBridge.exportCSVResult", payload.ReqId, nil, err)

			case "insertRow":
				res, err := activeDriver.InsertRow(payload.TableName, payload.Data)
				if err == nil {
					b.Broadcast("dbBridge.insertRowResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": true, "row": res,
					})
				} else {
					handleResult(b, "dbBridge.insertRowResult", payload.ReqId, nil, err)
				}

			case "deleteRows":
				err := activeDriver.DeleteRows(payload.TableName, payload.PKColumn, payload.PKValues)
				handleResult(b, "dbBridge.deleteRowsResult", payload.ReqId, nil, err)

			case "getTableColumns":
				res, err := activeDriver.GetTableColumns(payload.TableName)
				if err == nil {
					b.Broadcast("dbBridge.getTableColumnsResult", map[string]interface{}{
						"reqId": payload.ReqId, "success": true, "columns": res,
					})
				} else {
					handleResult(b, "dbBridge.getTableColumnsResult", payload.ReqId, nil, err)
				}

			case "alterTable":
				err := activeDriver.AlterTable(payload.TableName, payload.Operations)
				handleResult(b, "dbBridge.alterTableResult", payload.ReqId, nil, err)

			case "updateExtension":
				err := updateSelf(payload.DownloadUrl)
				handleResult(b, "dbBridge.updateExtensionResult", payload.ReqId, nil, err)
			}
		}()
	})
}

func updateSelf(url string) error {
	log.Printf("Starting extension update from: %s\n", url)
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	exePath, err := os.Executable()
	if err != nil {
		return err
	}

	oldPath := exePath + ".old"
	// Rename current exe to .old (works on Windows while running)
	// Ignore error if .old already exists or use Remove first
	os.Remove(oldPath)
	if err := os.Rename(exePath, oldPath); err != nil {
		return err
	}

	out, err := os.Create(exePath)
	if err != nil {
		// Try to restore if create fails
		os.Rename(oldPath, exePath)
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return err
	}

	// On Unix-like systems, ensure it's executable
	os.Chmod(exePath, 0755)

	log.Println("Extension updated successfully. Ready for restart.")
	return nil
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
		// Based on the frontend:
		// schema.ts expects payload.schema
		// grid.ts expects payload.rows, payload.fields directly (flattened)
		
		if strings.HasPrefix(event, "dbBridge.get") {
			key := strings.TrimPrefix(event, "dbBridge.get")
			key = strings.TrimSuffix(key, "Result")
			// Lowercase the first letter
			key = strings.ToLower(key[:1]) + key[1:]
			resp[key] = res
		} else {
			// Flatten the result into resp
			data, _ := json.Marshal(res)
			json.Unmarshal(data, &resp)
		}
	}
	b.Broadcast(event, resp)
}
