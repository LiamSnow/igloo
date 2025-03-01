export async function command(cmd) {
    const response = await fetch('http://localhost:3000', {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: cmd
    });

    if (!response.ok) {
        console.error(`API error: ${response.status}`);
        return false;
    }

    return true;
}

export async function fetchUIData() {
    const response = await fetch('http://localhost:3000', {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: 'ui get'
    });

    if (!response.ok) {
        console.error(`API error: ${response.status}`);
        return null;
    }

    return await response.json();
}
