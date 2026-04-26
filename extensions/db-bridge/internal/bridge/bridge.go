package bridge

import (
	"encoding/json"
	"fmt"
	"log"
	"net/url"
	"os"
	"sync"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
)

type AuthInfo struct {
	NLPort         string `json:"nlPort"`
	NLToken        string `json:"nlToken"`
	NLConnectToken string `json:"nlConnectToken"`
	NLExtId        string `json:"nlExtensionId"`
}

type Message struct {
	ID          string          `json:"id,omitempty"`
	Method      string          `json:"method,omitempty"`
	AccessToken string          `json:"accessToken,omitempty"`
	Event       string          `json:"event,omitempty"`
	Data        json.RawMessage `json:"data,omitempty"`
}

type ResponsePayload struct {
	Event string      `json:"event"`
	Data  interface{} `json:"data"`
}

type Bridge struct {
	auth AuthInfo
	ws   *websocket.Conn
	mu   sync.Mutex
}

func NewBridge(auth AuthInfo) *Bridge {
	return &Bridge{auth: auth}
}

func (b *Bridge) Connect() error {
	u := url.URL{
		Scheme: "ws",
		Host:   fmt.Sprintf("localhost:%s", b.auth.NLPort),
		RawQuery: fmt.Sprintf("extensionId=%s&connectToken=%s", 
			url.QueryEscape(b.auth.NLExtId), 
			url.QueryEscape(b.auth.NLConnectToken)),
	}

	log.Printf("Connecting to WebSocket: %s\n", u.String())

	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		log.Printf("WebSocket dial error: %v\n", err)
		return err
	}

	b.ws = c
	return nil
}

func (b *Bridge) Broadcast(event string, data interface{}) error {
	b.mu.Lock()
	defer b.mu.Unlock()

	if b.ws == nil {
		log.Println("Broadcast failed: websocket not connected")
		return fmt.Errorf("websocket not connected")
	}

	payload := ResponsePayload{
		Event: event,
		Data:  data,
	}

	log.Printf("Broadcasting event: %s\n", event)

	msg := Message{
		ID:          uuid.New().String(),
		Method:      "app.broadcast",
		AccessToken: b.auth.NLToken,
		Data:        nil, // We need to encode the payload into Data
	}

	encodedPayload, err := json.Marshal(payload)
	if err != nil {
		return err
	}
	msg.Data = encodedPayload

	return b.ws.WriteJSON(msg)
}

func (b *Bridge) Listen(handler func(Message)) {
	for {
		_, message, err := b.ws.ReadMessage()
		if err != nil {
			log.Println("read:", err)
			return
		}

		var msg Message
		if err := json.Unmarshal(message, &msg); err != nil {
			log.Println("unmarshal:", err)
			continue
		}

		// Handle windowClose to exit gracefully
		if msg.Event == "windowClose" {
			os.Exit(0)
		}

		handler(msg)
	}
}

func (b *Bridge) Close() {
	if b.ws != nil {
		b.ws.Close()
	}
}
