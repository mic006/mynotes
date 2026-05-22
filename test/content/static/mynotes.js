// MyNotes specific handling

/// Report to server a change of checkbox
async function eventCheckboxChange() {
    const checkbox = event.target;
    const new_state = checkbox.checked;

    console.log("[CB]:", new_state)
}

/// Manage checkbox inputs
function setCheckboxInputs() {
    const checkboxes = document.querySelectorAll('input[type="checkbox"]');
    checkboxes.forEach(checkbox => {
        checkbox.addEventListener('change', eventCheckboxChange);
    });
}

setCheckboxInputs();