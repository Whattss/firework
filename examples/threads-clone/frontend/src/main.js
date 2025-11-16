console.log('🔥 Threads Clone Frontend');
const app = document.getElementById('app');

async function loadThreads() {
    const res = await fetch('/api/threads');
    const threads = await res.json();
    
    app.innerHTML = `
        <h1>🔥 Threads Clone</h1>
        ${threads.map(t => `
            <div style="border:1px solid #ccc; padding:10px; margin:10px 0;">
                <b>${t.avatar} ${t.author}</b> ${t.handle}<br>
                ${t.content}<br>
                <small>❤️ ${t.likes} | 💬 ${t.replies} | 🔄 ${t.reposts}</small>
            </div>
        `).join('')}
    `;
}

loadThreads();
