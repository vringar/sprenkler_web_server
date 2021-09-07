'use strict';

function deleteSchedule(day, start_time, end_time) {
    let request = new Request(document.documentURI +`/timetable`,
        {
            method: 'DELETE',
            headers: {
                "Content-Type" : "application/json"
            },
            referrerPolicy: 'no-referrer',
            body: JSON.stringify({day, start_time, end_time})
        })
    fetch(request)
        .then(() => window.location.reload())
        .catch((e) => console.log(e))
}

document.addEventListener('DOMContentLoaded', (_event) => {
    for (let button of document.getElementsByClassName("schedule_delete_button")) {
        button.addEventListener("click", (elem, _ev) => {
            deleteSchedule(button.dataset.day, button.dataset.begin, button.dataset.end)
        })
    }
});