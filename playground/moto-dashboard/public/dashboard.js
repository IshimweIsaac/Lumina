let currentSelected = null;
const fleetGrid = document.getElementById('fleetContainer');
const activityFeed = document.getElementById('activityFeed');
const bikeDots = new Map(); // motoName -> element

// Initialize 1,000 dots
function initFleet() {
    fleetGrid.innerHTML = '';
    for (let i = 1; i <= 1000; i++) {
        const dot = document.createElement('div');
        const name = `moto${i}`;
        dot.className = 'bike-dot';
        dot.id = `dot-${name}`;
        dot.onclick = () => selectBike(name);
        fleetGrid.appendChild(dot);
        bikeDots.set(name, dot);
    }
}

async function updateState() {
    try {
        const res = await fetch('/api/state');
        const data = await res.json();
        
        if (data.instances) {
            updateFleetUI(data.instances);
            updateStats(data.instances);
        }
        
        if (data.messages && data.messages.length > 0) {
            addActivityMessages(data.messages);
        }

        if (currentSelected) {
            updateDetailView(data.instances[currentSelected]);
        }
    } catch (e) {
        console.error("Poll error:", e);
    }
}

function updateFleetUI(instances) {
    for (const [name, data] of Object.entries(instances)) {
        const dot = bikeDots.get(name);
        if (!dot) continue;

        const fields = data.fields;
        dot.className = 'bike-dot';
        if (fields.isLocked) dot.classList.add('locked');
        else if (fields.isCharging) dot.classList.add('charging');
        else if (fields.isBusy) dot.classList.add('in-use');
        else dot.classList.add('available');

        if (fields.isLowBattery) dot.classList.add('low-battery');
        
        dot.style.setProperty('color', getComputedStyle(dot).backgroundColor);
    }
}

function updateStats(instances) {
    let available = 0, inUse = 0, locked = 0;
    for (const inst of Object.values(instances)) {
        if (inst.fields.isLocked) locked++;
        else if (inst.fields.isBusy) inUse++;
        else available++;
    }
    document.getElementById('availableCount').innerText = available.toLocaleString();
    document.getElementById('inUseCount').innerText = inUse.toLocaleString();
    document.getElementById('lockedCount').innerText = locked.toLocaleString();
}

function addActivityMessages(messages) {
    messages.forEach(msg => {
        const item = document.createElement('div');
        const isAlert = msg.toLowerCase().includes('alert') || msg.toLowerCase().includes('low');
        item.className = `activity-item ${isAlert ? 'alert' : ''}`;
        
        const time = new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
        item.innerHTML = `<span class="activity-timestamp">[${time}]</span> ${msg}`;
        
        activityFeed.prepend(item);
        if (activityFeed.children.length > 50) {
            activityFeed.removeChild(activityFeed.lastChild);
        }
    });
    
    const countBadge = document.getElementById('eventCount');
    countBadge.innerText = `${messages.length} New`;
    countBadge.style.opacity = '1';
    setTimeout(() => countBadge.style.opacity = '0.7', 2000);
}

function selectBike(name) {
    currentSelected = name;
    document.getElementById('detailOverlay').classList.add('open');
    document.getElementById('detailName').innerText = name.toUpperCase();
    updateState(); // Refresh immediately
}

function updateDetailView(data) {
    if (!data) return;
    const f = data.fields;
    document.getElementById('detailStatus').innerText = f.status.toUpperCase();
    document.getElementById('batteryFill').style.width = `${f.battery}%`;
    document.getElementById('batteryValue').innerText = `${Math.round(f.battery)}%`;
    document.getElementById('detailStatusText').innerText = f.isLocked ? "Locked" : "Unlocked";
    document.getElementById('isChargingText').innerText = f.isCharging ? "Active" : "None";
}

// Controls
document.getElementById('simulateBtn').onclick = async () => {
    // Pick 50 random bikes
    const count = 50;
    for(let i=0; i<count; i++) {
        const id = Math.floor(Math.random() * 1000) + 1;
        const name = `moto${id}`;
        const isStart = Math.random() > 0.5;
        
        await fetch('/api/event', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ instance: name, field: 'isBusy', value: isStart })
        });
        
        // Random battery drain
        if (isStart) {
            await fetch('/api/event', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ instance: name, field: 'battery', value: Math.max(0, 80 - Math.random() * 20) })
            });
        }
    }
};

document.getElementById('chargeAllBtn').onclick = async () => {
    for(let i=1; i<=1000; i++) {
        const name = `moto${i}`;
        // Only charge if not busy
        await fetch('/api/event', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ instance: name, field: 'isCharging', value: true })
        });
    }
};

document.getElementById('resetBtn').onclick = async () => {
    if (confirm("Reset all 1,000 bikes to initial state?")) {
        location.reload();
    }
};

document.getElementById('closeOverlay').onclick = () => {
    document.getElementById('detailOverlay').classList.remove('open');
    currentSelected = null;
};

// Start
initFleet();
setInterval(updateState, 1000);
updateState();
