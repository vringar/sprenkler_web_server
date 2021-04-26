'use strict';
function updateStatus(valve_number, new_status) {
    let request = new Request(`/valves/${valve_number}/status`,
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

function deleteButton(valve_number) {
    let request = new Request(`/valves/${valve_number}/`,
        {
            method: 'DELETE',
            referrerPolicy: 'no-referrer',
        })
    fetch(request)
        .then(() => window.location.reload())
        .catch((e) => console.log(e))
}

document.addEventListener('DOMContentLoaded', (event) => {
    for (let radioButton of document.getElementsByClassName("automation_status_radio")) {
        let valve_number = radioButton.dataset.valve_number;
        let value = radioButton.value;
        radioButton.addEventListener("click", (elem, ev) => { updateStatus(valve_number, value) })
    }
    for (let button of document.getElementsByClassName("valve_delete_button")) {

        button.addEventListener("click", (elem, ev) => deleteButton(button.dataset.valve_number) )
    }
});