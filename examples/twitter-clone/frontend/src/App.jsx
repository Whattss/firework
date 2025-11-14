import { useState, useEffect } from 'react';
import './App.css';

function App() {
  const [tweets, setTweets] = useState([]);
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetchTweets();
  }, []);

  const fetchTweets = async () => {
    const res = await fetch('/api/tweets');
    const data = await res.json();
    setTweets(data);
  };

  const postTweet = async (e) => {
    e.preventDefault();
    if (!content.trim() || content.length > 280) return;
    
    setLoading(true);
    await fetch('/api/tweets', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content })
    });
    setContent('');
    await fetchTweets();
    setLoading(false);
  };

  const toggleLike = async (id) => {
    await fetch(`/api/tweets/${id}/like`, { method: 'POST' });
    await fetchTweets();
  };

  const toggleRetweet = async (id) => {
    await fetch(`/api/tweets/${id}/retweet`, { method: 'POST' });
    await fetchTweets();
  };

  const formatTime = (timestamp) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = Math.floor((now - date) / 1000);
    
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  };

  return (
    <div className="app">
      <header className="header">
        <h1>🔥</h1>
      </header>

      <div className="container">
        <form className="compose" onSubmit={postTweet}>
          <textarea
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="What's happening?"
            maxLength={280}
            disabled={loading}
          />
          <div className="compose-footer">
            <span className={content.length > 280 ? 'count error' : 'count'}>
              {content.length}/280
            </span>
            <button type="submit" disabled={!content.trim() || loading}>
              Tweet
            </button>
          </div>
        </form>

        <div className="timeline">
          {tweets.map(tweet => (
            <div key={tweet.id} className="tweet">
              <div className="tweet-header">
                <span className="avatar">{tweet.avatar}</span>
                <div className="tweet-info">
                  <span className="author">{tweet.author}</span>
                  <span className="handle">{tweet.handle}</span>
                  <span className="time">· {formatTime(tweet.timestamp)}</span>
                </div>
              </div>
              <div className="tweet-content">{tweet.content}</div>
              <div className="tweet-actions">
                <button className="action">
                  💬 {tweet.replies > 0 && tweet.replies}
                </button>
                <button 
                  className={tweet.retweeted ? 'action active' : 'action'}
                  onClick={() => toggleRetweet(tweet.id)}
                >
                  🔁 {tweet.retweets > 0 && tweet.retweets}
                </button>
                <button 
                  className={tweet.liked ? 'action active' : 'action'}
                  onClick={() => toggleLike(tweet.id)}
                >
                  {tweet.liked ? '❤️' : '��'} {tweet.likes > 0 && tweet.likes}
                </button>
                <button className="action">📊</button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export default App;
