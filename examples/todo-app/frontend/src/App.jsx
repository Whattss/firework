import { useState, useEffect } from 'react'
import './App.css'

function App() {
  const [todos, setTodos] = useState([])
  const [input, setInput] = useState('')

  useEffect(() => {
    fetch('/api/todos').then(r => r.json()).then(setTodos)
  }, [])

  const addTodo = async (e) => {
    e.preventDefault()
    if (!input.trim()) return
    const res = await fetch('/api/todos', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text: input }),
    })
    setTodos([...todos, await res.json()])
    setInput('')
  }

  const toggleTodo = async (id) => {
    const res = await fetch(`/api/todos/${id}`, { method: 'PATCH' })
    setTodos(todos.map(t => t.id === id ? await res.json() : t))
  }

  const deleteTodo = async (id) => {
    await fetch(`/api/todos/${id}`, { method: 'DELETE' })
    setTodos(todos.filter(t => t.id !== id))
  }

  return (
    <div className="app">
      <div className="container">
        <h1>🔥 Firework TODO</h1>
        <p className="subtitle">Firework + Vite + React</p>
        
        <form onSubmit={addTodo} className="todo-form">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="What needs to be done?"
            className="todo-input"
          />
          <button type="submit" className="add-btn">Add</button>
        </form>

        <div className="todos">
          {todos.length === 0 ? (
            <p className="empty">No todos yet!</p>
          ) : (
            todos.map(todo => (
              <div key={todo.id} className={`todo-item ${todo.completed ? 'completed' : ''}`}>
                <input
                  type="checkbox"
                  checked={todo.completed}
                  onChange={() => toggleTodo(todo.id)}
                />
                <span>{todo.text}</span>
                <button onClick={() => deleteTodo(todo.id)}>🗑️</button>
              </div>
            ))
          )}
        </div>

        <div className="stats">
          {todos.length} items • {todos.filter(t => !t.completed).length} active
        </div>
      </div>
    </div>
  )
}

export default App
