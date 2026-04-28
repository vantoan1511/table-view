// Package pool provides a thread-safe LRU connection pool for database drivers.
// It maintains up to MaxSize concurrent live connections keyed by connectionId.
// When the pool is full, the least-recently-used connection is closed and evicted.
package pool

import (
	"container/list"
	"log"
	"sync"

	"github.com/vanto/table-view/db-bridge/internal/drivers"
)

const MaxSize = 5

// entry holds a driver and its LRU list element.
type entry struct {
	id     string
	driver drivers.DatabaseDriver
	elem   *list.Element
}

// Pool is a thread-safe LRU connection pool.
type Pool struct {
	mu      sync.Mutex
	cache   map[string]*entry
	lruList *list.List // front = most recently used
}

// New creates an empty Pool.
func New() *Pool {
	return &Pool{
		cache:   make(map[string]*entry),
		lruList: list.New(),
	}
}

// Get returns the driver for the given connectionId if it exists, updating its LRU position.
// Returns nil if not found.
func (p *Pool) Get(id string) drivers.DatabaseDriver {
	p.mu.Lock()
	defer p.mu.Unlock()

	if e, ok := p.cache[id]; ok {
		p.lruList.MoveToFront(e.elem)
		return e.driver
	}
	return nil
}

// Put registers a driver under the given connectionId.
// If the pool is already at MaxSize, the LRU entry is evicted first.
// If an entry for id already exists it is replaced (old driver is disconnected).
func (p *Pool) Put(id string, d drivers.DatabaseDriver) {
	p.mu.Lock()
	defer p.mu.Unlock()

	// Replace existing
	if e, ok := p.cache[id]; ok {
		if err := e.driver.Disconnect(); err != nil {
			log.Printf("pool: error disconnecting replaced driver %s: %v", id, err)
		}
		p.lruList.Remove(e.elem)
		delete(p.cache, id)
	}

	// Evict LRU if at capacity
	if len(p.cache) >= MaxSize {
		oldest := p.lruList.Back()
		if oldest != nil {
			ev := oldest.Value.(*entry)
			log.Printf("pool: evicting LRU connection %s (pool full)", ev.id)
			if err := ev.driver.Disconnect(); err != nil {
				log.Printf("pool: error disconnecting evicted driver %s: %v", ev.id, err)
			}
			p.lruList.Remove(oldest)
			delete(p.cache, ev.id)
		}
	}

	elem := p.lruList.PushFront(nil) // placeholder
	e := &entry{id: id, driver: d, elem: elem}
	elem.Value = e
	p.cache[id] = e
}

// Remove explicitly closes and removes a connection from the pool.
func (p *Pool) Remove(id string) {
	p.mu.Lock()
	defer p.mu.Unlock()

	if e, ok := p.cache[id]; ok {
		if err := e.driver.Disconnect(); err != nil {
			log.Printf("pool: error disconnecting removed driver %s: %v", id, err)
		}
		p.lruList.Remove(e.elem)
		delete(p.cache, id)
	}
}

// CloseAll disconnects every active driver in the pool.
func (p *Pool) CloseAll() {
	p.mu.Lock()
	defer p.mu.Unlock()

	for id, e := range p.cache {
		if err := e.driver.Disconnect(); err != nil {
			log.Printf("pool: error closing driver %s: %v", id, err)
		}
		p.lruList.Remove(e.elem)
	}
	p.cache = make(map[string]*entry)
}
