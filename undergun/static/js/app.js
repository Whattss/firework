const API_URL = 'http://localhost:8080/api';

function getToken() {
    return localStorage.getItem('token');
}

function isLoggedIn() {
    return !!getToken();
}

async function fetchPosts() {
    try {
        const response = await fetch(`${API_URL}/posts/`);
        const posts = await response.json();
        displayPosts(posts);
    } catch (error) {
        console.error('Error fetching posts:', error);
        document.getElementById('postsContainer').innerHTML = '<p class="error">Failed to load posts</p>';
    }
}

function displayPosts(posts) {
    const container = document.getElementById('postsContainer');
    
    if (posts.length === 0) {
        container.innerHTML = '<p style="color: white; text-align: center;">No posts yet. Be the first to post!</p>';
        return;
    }

    container.innerHTML = posts.map(post => `
        <div class="post">
            <div class="post-header">
                <div>
                    <div class="post-author">@${post.author.username}</div>
                    <div class="post-date">${new Date(post.created_at).toLocaleString()}</div>
                </div>
            </div>
            <h3 class="post-title">${escapeHtml(post.title)}</h3>
            <p class="post-content">${escapeHtml(post.content)}</p>
            ${canDeletePost(post) ? `
                <div class="post-actions">
                    <button class="delete-btn" onclick="deletePost(${post.id})">Delete</button>
                </div>
            ` : ''}
        </div>
    `).join('');
}

function canDeletePost(post) {
    const token = getToken();
    if (!token) return false;
    
    try {
        const payload = JSON.parse(atob(token.split('.')[1]));
        return payload.sub === post.author.id;
    } catch {
        return false;
    }
}

async function deletePost(postId) {
    if (!confirm('Are you sure you want to delete this post?')) return;

    try {
        const response = await fetch(`${API_URL}/posts/${postId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${getToken()}`
            }
        });

        if (response.ok) {
            fetchPosts();
        } else {
            alert('Failed to delete post');
        }
    } catch (error) {
        console.error('Error deleting post:', error);
        alert('Failed to delete post');
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function updateNav() {
    const loginLink = document.getElementById('loginLink');
    const registerLink = document.getElementById('registerLink');
    const logoutLink = document.getElementById('logoutLink');
    const userInfo = document.getElementById('userInfo');
    const createPostSection = document.getElementById('createPostSection');

    if (isLoggedIn()) {
        const token = getToken();
        const payload = JSON.parse(atob(token.split('.')[1]));
        
        loginLink.style.display = 'none';
        registerLink.style.display = 'none';
        logoutLink.style.display = 'block';
        userInfo.style.display = 'block';
        userInfo.textContent = `@${payload.username}`;
        
        if (createPostSection) {
            createPostSection.style.display = 'block';
        }
    } else {
        loginLink.style.display = 'block';
        registerLink.style.display = 'block';
        logoutLink.style.display = 'none';
        userInfo.style.display = 'none';
        
        if (createPostSection) {
            createPostSection.style.display = 'none';
        }
    }
}

document.addEventListener('DOMContentLoaded', () => {
    updateNav();
    fetchPosts();

    const logoutLink = document.getElementById('logoutLink');
    if (logoutLink) {
        logoutLink.addEventListener('click', (e) => {
            e.preventDefault();
            localStorage.removeItem('token');
            window.location.href = '/';
        });
    }

    const createPostForm = document.getElementById('createPostForm');
    if (createPostForm) {
        createPostForm.addEventListener('submit', async (e) => {
            e.preventDefault();

            const title = document.getElementById('postTitle').value;
            const content = document.getElementById('postContent').value;

            try {
                const response = await fetch(`${API_URL}/posts/`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${getToken()}`
                    },
                    body: JSON.stringify({ title, content })
                });

                if (response.ok) {
                    document.getElementById('postTitle').value = '';
                    document.getElementById('postContent').value = '';
                    fetchPosts();
                } else {
                    alert('Failed to create post');
                }
            } catch (error) {
                console.error('Error creating post:', error);
                alert('Failed to create post');
            }
        });
    }
});
