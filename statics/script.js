const expandedGroups = new Set();

async function fetchTasks() {
    const res = await fetch('/api/jobs');
    const tasks = await res.json();
    const tbody = document.querySelector('#task-table tbody');
    tbody.innerHTML = '';

    const filterText = document.getElementById('filter-input').value.toLowerCase();
    const filtered = tasks.filter(t =>
        t.name.toLowerCase().includes(filterText) ||
        t.task_type.toLowerCase().includes(filterText)
    );

    document.getElementById('task-count').textContent = `${filtered.length} tasks`;

    const grouped = filtered.reduce((acc, task) => {
        acc[task.name] = acc[task.name] || [];
        acc[task.name].push(task);
        return acc;
    }, {});

    const statusMap = {
        start: '🟡 Start',
        scheduled: '📅 Scheduled',
        running: '🔄 Running',
        success: '✅ Success',
        failed: '❌ Failed',
        disabled: '🚫 Disabled',
    };

    Object.entries(grouped).forEach(([name, jobs], idx) => {
        const toggleId = `group-${idx}`;
        const isOpen = expandedGroups.has(toggleId);

        tbody.insertAdjacentHTML('beforeend', `
            <tr>
                <td colspan="6">
                    <strong>${name}</strong>
                    <button onclick="toggleGroup('${toggleId}', this)">
                        ${isOpen ? '▼' : '▶'}
                    </button>
                </td>
            </tr>
        `);

        jobs.sort((a, b) => new Date(b.last_run || 0) - new Date(a.last_run || 0))
            .forEach(task => {
                const status = statusMap[task.status?.toLowerCase()] || task.status || 'Unknown';
                tbody.insertAdjacentHTML('beforeend', `
                    <tr class="${toggleId}" style="display: ${isOpen ? '' : 'none'};">
                        <td></td>
                        <td>${task.task_type}</td>
                        <td><span class="status">${status}</span></td>
                        <td>${task.last_run || '-'}</td>
                        <td><pre>${JSON.stringify(task.payload, null, 2)}</pre></td>
                        <td>${task.execution_count || 0}</td>
                    </tr>
                `);
            });
    });
}

function toggleGroup(groupClass, btn) {
    const rows = document.querySelectorAll(`.${groupClass}`);
    const shouldShow = rows[0]?.style.display === 'none';

    rows.forEach(r => r.style.display = shouldShow ? '' : 'none');
    btn.textContent = shouldShow ? '▼' : '▶';

    if (shouldShow) expandedGroups.add(groupClass);
    else expandedGroups.delete(groupClass);
}

function setupFilterBar() {
    document.querySelector('main').insertAdjacentHTML('afterbegin', `
        <div style="margin-bottom: 1rem; display: flex; justify-content: space-between; align-items: center;">
            <input id="filter-input" type="text" placeholder="Filter by name or type..." style="padding: 0.5rem; width: 300px; font-size: 1rem;">
            <span id="task-count" style="font-weight: bold;"></span>
        </div>
    `);
    document.getElementById('filter-input').addEventListener('input', fetchTasks);
}

setupFilterBar();
fetchTasks();
setInterval(fetchTasks, 10000);