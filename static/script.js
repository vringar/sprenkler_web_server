async function updateStatus(index) {
    let request = new Request(`/valves/${i}/toggle`)
    await fetch(request, {method: "POST"})
}