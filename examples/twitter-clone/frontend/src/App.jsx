import { useState, useEffect } from 'react'
import './App.css'

function App() {
  const [tweets, setTweets] = useState([])
  const [newTweet, setNewTweet] = useState('')
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    fetchTweets()
  }, [])

  const fetchTweets = async () => {
    setLoading(true)
    try {
      const res = await fetch('/api/tweets')
      const data = await res.json()
      setTweets(data)
    } catch (err) {
      console.error('Failed to fetch tweets:', err)
    } finally {
      setLoading(false)
    }
  }

  const postTweet = async () => {
    if (!newTweet.trim()) return
    
    try {
      const res = await fetch('/api/tweets', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: newTweet })
      })
      const tweet = await res.json()
      setTweets([tweet, ...tweets])
      setNewTweet('')
    } catch (err) {
      console.error('Failed to post tweet:', err)
    }
  }

  const toggleLike = async (id) => {
    try {
      const res = await fetch(`/api/tweets/${id}/like`, { method: 'POST' })
      const updated = await res.json()
      setTweets(tweets.map(t => t.id === id ? updated : t))
    } catch (err) {
      console.error('Failed to like tweet:', err)
    }
  }

  const toggleRetweet = async (id) => {
    try {
      const res = await fetch(`/api/tweets/${id}/retweet`, { method: 'POST' })
      const updated = await res.json()
      setTweets(tweets.map(t => t.id === id ? updated : t))
    } catch (err) {
      console.error('Failed to retweet:', err)
    }
  }

  const formatTime = (timestamp) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = Math.floor((now - date) / 1000)
    if (diff < 60) return `${diff}s`
    if (diff < 3600) return `${Math.floor(diff / 60)}m`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`
    return `${Math.floor(diff / 86400)}d`
  }

  const handleKeyDown = (e) => {
    if (e.ctrlKey && e.key === 'Enter') {
      postTweet()
    }
  }

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="logo">🔥</div>
        <nav className="nav">
          <a href="#" className="nav-item active">
            <span className="icon">🏠</span>
            <span>Home</span>
          </a>
          <a href="#" className="nav-item">
            <span className="icon">🔍</span>
            <span>Explore</span>
          </a>
          <a href="#" className="nav-item">
            <span className="icon">🔔</span>
            <span>Notifications</span>
          </a>
          <a href="#" className="nav-item">
            <span className="icon">👤</span>
            <span>Profile</span>
          </a>
        </nav>
        <button className="tweet-btn">Post</button>
      </aside>

      <main className="main">
        <div className="header">
          <h1>Home</h1>
        </div>

        <div className="composer">
          <div className="avatar">👤</div>
          <div className="composer-input">
            <textarea
              value={newTweet}
              onChange={(e) => setNewTweet(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="What's happening?!"
              maxLength={280}
            />
            <div className="composer-actions">
              <span className={`char-count ${newTweet.length > 260 ? 'warning' : ''}`}>
                {newTweet.length}/280
              </span>
              <button
                onClick={postTweet}
                disabled={!newTweet.trim()}
                className="post-btn"
              >
                Post
              </button>
            </div>
          </div>
        </div>

        <div className="timeline">
          {loading && <div className="loading">Loading...</div>}
          
          {tweets.map((tweet) => (
            <article key={tweet.id} className="tweet">
              <div className="tweet-avatar">{tweet.avatar}</div>
              <div className="tweet-content">
                <div className="tweet-header">
                  <span className="author">{tweet.author}</span>
                  <span className="handle">{tweet.handle}</span>
                  <span className="timestamp">· {formatTime(tweet.timestamp)}</span>
                </div>
                <p className="tweet-text">{tweet.content}</p>
                <div className="tweet-actions">
                  <button className="action">
                    <span className="icon">💬</span>
                    {tweet.replies > 0 && <span>{tweet.replies}</span>}
                  </button>
                  <button
                    className={`action ${tweet.retweeted ? 'active' : ''}`}
                    onClick={() => toggleRetweet(tweet.id)}
                  >
                    <span className="icon">🔄</span>
                    {tweet.retweets > 0 && <span>{tweet.retweets}</span>}
                  </button>
                  <button
                    className={`action ${tweet.liked ? 'active' : ''}`}
                    onClick={() => toggleLike(tweet.id)}
                  >
                    <span className="icon">{tweet.liked ? '❤️' : '🤍'}</span>
                    {tweet.likes > 0 && <span>{tweet.likes}</span>}
                  </button>
                </div>
              </div>
            </article>
          ))}
        </div>
      </main>

      <aside className="widgets">
        <div className="widget">
          <h2>What's happening</h2>
          <div className="trend">
            <span className="category">Technology · Trending</span>
            <h3>Firework Framework</h3>
            <span className="count">12.5K posts</span>
          </div>
          <div className="trend">
            <span className="category">Programming · Trending</span>
            <h3>Rust Web Development</h3>
            <span className="count">8.2K posts</span>
          </div>
          <div className="trend">
            <span className="category">Tech · Trending</span>
            <h3>Fullstack Rust</h3>
            <span className="count">5.7K posts</span>
          </div>
        </div>

        <div className="widget">
          <h2>Who to follow</h2>
          <div className="follow-item">
            <div className="follow-avatar">🦀</div>
            <div className="follow-info">
              <div className="follow-name">Rust Foundation</div>
              <div className="follow-handle">@rust_foundation</div>
            </div>
            <button className="follow-btn">Follow</button>
          </div>
          <div className="follow-item">
            <div className="follow-avatar">⚡</div>
            <div className="follow-info">
              <div className="follow-name">Vite</div>
              <div className="follow-handle">@vite_js</div>
            </div>
            <button className="follow-btn">Follow</button>
          </div>
        </div>
      </aside>
    </div>
  )
}

export default App
