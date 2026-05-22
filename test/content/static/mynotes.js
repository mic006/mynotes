// MyNotes specific handling

/// Report to server a change of checkbox
async function eventCheckboxChange() {
    const checkbox = event.target;
    const new_state = checkbox.checked;
    try {
        const res = await fetch(checkbox.dataset.url, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                state: new_state,
                label: checkbox.dataset.label
            })
        });

        if (!res.ok) {
            throw new Error("Checkbox change failed with status " + res.status);
        }
        console.log("[Checkbox]: '" + checkbox.dataset.label + "' state changed to", new_state);
    } catch (err) {
        console.error(err);
        // revert checkbox state as server has not been updated
        checkbox.checked = !new_state;
    }
}

/// Manage checkbox inputs
function setCheckboxInputs() {
    const checkboxes = document.querySelectorAll('input[type="checkbox"]');
    checkboxes.forEach(checkbox => {
        checkbox.addEventListener('change', eventCheckboxChange);
    });
}

setCheckboxInputs();