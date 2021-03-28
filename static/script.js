function updateStatus(index, new_status) {
    let request = new Request(`/valves/${index}/status`,
        {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            referrerPolicy: 'no-referrer',
            body: JSON.stringify(new_status)
        })
    fetch(request)
        .then(() => window.location.reload())
        .catch((e) => console.log(e))
}

function deleteButton(index) {
    let request = new Request(`/valves/${index}/`,
        {
            method: 'DELETE',
            referrerPolicy: 'no-referrer',
        })
    fetch(request)
        .then(() => window.location.reload())
        .catch((e) => console.log(e))
}

document.addEventListener('DOMContentLoaded', (event) => {
    for (let form of document.getElementsByClassName("automation_status_form")) {
        let index = form.dataset.index;
        for (let radioButton of form.getElementsByTagName("input")) {
            let value = radioButton.value;
            radioButton.addEventListener("click", (elem, ev) => { updateStatus(index, value) })
        }
    }
    for (let button of document.getElementsByClassName("valve_delete_button")) {

        button.addEventListener("click", (elem, ev) => deleteButton(button.dataset.index) )
    }
});