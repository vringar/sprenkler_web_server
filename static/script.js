function updateStatus(index, new_status) {
    let request = new Request(`/valves/${index}/status`,
    {
        method: 'POST', // *GET, POST, PUT, DELETE, etc.
        mode: 'cors', // no-cors, *cors, same-origin
        cache: 'no-cache', // *default, no-cache, reload, force-cache, only-if-cached
        credentials: 'same-origin', // include, *same-origin, omit
        headers: {
          'Content-Type': 'application/json'
          // 'Content-Type': 'application/x-www-form-urlencoded',
        },
        redirect: 'follow', // manual, *follow, error
        referrerPolicy: 'no-referrer', // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
        body: JSON.stringify(new_status) // body data type must match "Content-Type" header
      })
    fetch(request, {method: "POST"})
    .then(() => window.location.reload())
    .catch((e) => console.log(e))
}

document.addEventListener('DOMContentLoaded', (event) => {
    for (let form of document.getElementsByClassName("automation_status_form")) {
        let index = form.dataset.index;
        for (let radioButton of form.getElementsByTagName("input")) {
            let value = radioButton.value;
            radioButton.addEventListener("click", (elem, ev) => {updateStatus(index, value)})
        }
    }
});