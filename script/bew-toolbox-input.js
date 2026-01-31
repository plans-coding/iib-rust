// DOWNLOAD/SAVE/UPLOAD ----------

async function downloadTripDataFromOPFS() {
    try {
        const root = await navigator.storage.getDirectory();

        const fileHandle = await root.getFileHandle("tripData.json");
        const file = await fileHandle.getFile();

        const blob = await file.arrayBuffer();

        const url = URL.createObjectURL(
            new Blob([blob], { type: "application/json" })
        );

        const a = document.createElement("a");
        a.href = url;
        a.download = "tripData.json";
        document.body.appendChild(a);
        a.click();

        a.remove();
        URL.revokeObjectURL(url);

    } catch (err) {
        console.error("Download failed:", err);
        alert("tripData.json not found in OPFS");
    }
}

async function uploadTripDataToOPFS() {
    try {
        const [fileHandle] = await window.showOpenFilePicker({
            multiple: false,
            types: [{
                description: "JSON files",
                accept: { "application/json": [".json"] }
            }]
        });

        const file = await fileHandle.getFile();
        const data = await file.arrayBuffer();

        const root = await navigator.storage.getDirectory();

        const opfsHandle = await root.getFileHandle("tripData.json", { create: true });

        const writable = await opfsHandle.createWritable();
        await writable.write(data);   // replaces content
        await writable.close();

        console.log("tripData.json replaced in OPFS");

    } catch (err) {
        console.error("Upload failed:", err);
    }
}

async function removeTripDataFromOPFS() {
    try {
        // Ask user for confirmation
        const confirmDelete = confirm("Are you sure you want to delete tripData.json?");
        if (!confirmDelete) return; // exit if user cancels

        const root = await navigator.storage.getDirectory();
        await root.removeEntry("tripData.json");
        console.log("tripData.json removed from OPFS");
        alert("tripData.json has been deleted");

    } catch (err) {
        console.error("Remove failed:", err);
        alert("tripData.json does not exist in OPFS");
    }
}

// GENERATE SQL FROM JSON ----

function escapeSQLiteString(value) {
    if (value === null || value === undefined) return "NULL";
    return `'${value.toString().replace(/'/g, "''")}'`;
}

function objectToSQLiteInsert(tableName, obj) {
    const columns = Object.keys(obj).join(", ");
    const values = Object.values(obj).map(escapeSQLiteString).join(", ");
    return `INSERT INTO ${tableName} (${columns}) VALUES (${values});`;
}

async function generateSQLCode() {
    try {
        // Get the OPFS root directory
        const root = await navigator.storage.getDirectory();

        // Get the file handle for tripData.json
        const fileHandle = await root.getFileHandle("tripData.json");
        const file = await fileHandle.getFile();

        // Read the file as JSON
        const jsonData = JSON.parse(await file.text());

        // Convert overview to SQLite INSERT
        const overviewSQL = objectToSQLiteInsert("bewa_Overview", jsonData.overview);

        // Convert events to SQLite INSERTs
        const eventsSQL = jsonData.events.map(event =>
        objectToSQLiteInsert("bewb_Events", event)
        ).join("\n");

        const finalSQL = overviewSQL + "\n" + eventsSQL;

        //console.log(finalSQL);

        document.getElementById('generated_sql_code').textContent = finalSQL;
        document.getElementById('generated_sql_code').style.display = 'block';

    } catch (err) {
        console.error("Failed to read tripData.json:", err);
    }
}


// OTHER ----------

function copyPreviousDetails(btn, type) {
    const currentEvent = btn.closest(".event");
    if (!currentEvent) return;

    const prevEvent = currentEvent.previousElementSibling;
    if (!prevEvent) return;

    const fieldGroups = {
        AccommodationFields: [
            "Accommodation",
            "AccommodationCountry",
            "AccommodationCoordinatesAccuracy",
            "AccommodationCoordinates"
        ],
        TravelParticipants: [
            "TravelParticipants"
        ]
    };

    const fields = fieldGroups[type];
    if (!fields) return;

    for (const name of fields) {
        const source = prevEvent.querySelector(`[name="${name}"]`);
        const target = currentEvent.querySelector(`[name="${name}"]`);

        if (source && target) {
            target.value = source.value ?? "";
        }
    }
}



function checkDates() {

    const startDateInput = document.querySelector('input[name="DepartureDate"]');
    const endDateInput   = document.querySelector('input[name="ReturnDate"]');
    const dateStatusEl   = document.getElementById("current_trip_date_selection");

    if (!startDateInput || !endDateInput || !dateStatusEl) return;

    if (!startDateInput.value || !endDateInput.value) {
        dateStatusEl.textContent = "Select start and end date";
    } else {
        dateStatusEl.textContent = ""; // clear message
        document.getElementById("trip_summary_inputs").open = false;
    }
}

/* Also run once on page load */
checkDates();

let isDirty = false;
let saveTimer = null;

function setStatus(text) {
    const el = document.getElementById("current_trip_documentation_status");
    if (el) el.textContent = text;
}

function markDirty() {
    if (isDirty) return;

    isDirty = true;
    setStatus("Changes not saved");
}


/* debounce save (prevents writing on every keystroke) */
function scheduleSave() {
    markDirty();

    clearTimeout(saveTimer);

    saveTimer = setTimeout(async () => {
        await saveTrip2OPFS();
        isDirty = false;
        setStatus("Saved");
    }, 500); // 0.5s after last change
}

/* any input/textarea/select change triggers save */
document.addEventListener("input", scheduleSave);
document.addEventListener("change", scheduleSave);


/* attempt final save on tab close */
window.addEventListener("beforeunload", (e) => {
    if (!isDirty) return;

    // try save (best effort)
    saveTrip2OPFS();

    // show native browser warning
    e.preventDefault();
    e.returnValue = "";
});


const fileName = "tripData.json";
let fsRootHandle = null;


async function initOPFS() {
    fsRootHandle = await navigator.storage.getDirectory();
}

async function saveTrip2OPFS() {
    const data = buildJSONFromForm();

    const fileHandle = await fsRootHandle.getFileHandle(fileName, { create: true });
    const writable = await fileHandle.createWritable();

    await writable.write(JSON.stringify(data, null, 2));
    await writable.close();

    console.log("Trip saved to OPFS");
}

async function loadTripFromOPFS() {
    try {
        const fileHandle = await fsRootHandle.getFileHandle(fileName);
        const file = await fileHandle.getFile();
        return JSON.parse(await file.text());
    } catch {
        return null;
    }
}


function getDepartureDate() {
    return document.querySelector('input[name="DepartureDate"]')?.value;
}

function getReturnDate() {
    return document.querySelector('input[name="ReturnDate"]')?.value;
}

function daysBetween(startStr, endStr) {
    const start = new Date(startStr);
    const end = new Date(endStr);
    return Math.floor((end - start) / 86400000) + 1; // inclusive
}

function buildOverviewObject() {
    const overview = {};

    document.querySelectorAll("input, textarea, select").forEach(el => {
        if (!el.name) return;
        if (el.closest("#eventsFieldset")) return; // skip events

        overview[el.name] = el.value;
    });

    return overview;
}

function populateOverview(data) {
    document.querySelectorAll("input, textarea, select").forEach(el => {
        if (!el.name) return;
        if (el.closest("#eventsFieldset")) return;

        if (data[el.name] !== undefined) {
            el.value = data[el.name];
        }
    });
}

function createEventElement(dateStr, eventData = {}, index = 0) {
    const div = document.createElement("div");
    div.className = "event";
    div.style.border = "0px solid #ccc";
    div.style.padding = "0px";
    div.style.marginBottom = "5pt";

    div.innerHTML = `
    <details style="border-radius:10pt;border:3pt solid var(--color-hover-background);padding:3pt;user-select:none;">
        <summary style="padding:0 0 5pt 0;color:var(--color-hover-background);"><b>Day ${index}: ${dateStr}</b></summary>
        <div>
        <input type="hidden" name="Date" value="${dateStr}">
        </div>

        <label>Events <textarea type="text" class="form-control" name="Events" value="${eventData.Events || ""}"></textarea></label><br>

        <div style="display:flex;flex-direction:row;flex-wrap:wrap;gap:2pt;">
            <label style="flex:1;">Accommodation <input class="form-control" name="Accommodation" value="${eventData.Accommodation || ""}"></label><br>
            <label style="flex:1;">AccommodationCountry <input type="text" name="AccommodationCountry" value="${eventData.AccommodationCountry || ""}" class="form-control"></label><br>
            <label style="flex:1;">AccommodationCoordinatesAccuracy <input type="text" name="AccommodationCoordinatesAccuracy" value="${eventData.AccommodationCoordinatesAccuracy || ""}" class="form-control"></label><br>
            <label style="flex:1;">AccommodationCoordinates <input type="text" name="AccommodationCoordinates" value="${eventData.AccommodationCoordinates || ""}" class="form-control"></label><br>
            <button style="flex:1;" class="filter-button" type="button" onclick="copyPreviousDetails(this, 'AccommodationFields')">Copy</button>
        </div>

        <br>

        <div style="display:flex;flex-direction:column;flex-wrap:wrap;gap:5pt;">
            <label>TravelParticipants <input type="text" name="TravelParticipants" value="${eventData.TravelParticipants || ""}" class="form-control"></label>
            <button class="filter-button" style="flex:1;" type="button" onclick="copyPreviousDetails(this, 'TravelParticipants')">Copy</button><br>
        </div>


        <label>AdditionalNotes <input type="text" name="AdditionalNotes" value="${eventData.AdditionalNotes || ""}" class="form-control"></label>

        <br>

        <label>CountriesDuringDay <input type="text" name="CountriesDuringDay" value="${eventData.CountriesDuringDay || ""}" class="form-control"></label>
        <br>

    </details>
    `;

    return div;
}


function buildEventsArray() {
    const arr = [];

    document.querySelectorAll("#eventsFieldset .event").forEach(div => {
        const obj = {};

        div.querySelectorAll("input, textarea").forEach(el => {
            obj[el.name] = el.value;
        });

        arr.push(obj);
    });

    return arr;
}

function syncEventsToDateSpan(existingEvents = []) {
    const dep = getDepartureDate();
    const ret = getReturnDate();

    if (!dep || !ret) return;

    const wanted = daysBetween(dep, ret);

    const fs = document.getElementById("eventsFieldset");
    fs.innerHTML = "";

    const base = new Date(dep);

    for (let i = 0; i < wanted; i++) {
        const d = new Date(base);
        d.setDate(d.getDate() + i);

        const dateStr = d.toISOString().split("T")[0];

        fs.appendChild(
            createEventElement(dateStr, existingEvents[i] || {}, i)
        );
    }
}

function buildJSONFromForm() {
    return {
        overview: buildOverviewObject(),
        events: buildEventsArray()
    };
}

// INIT ---------------------------------

async function init_create_trip() {
    await initOPFS();

    const depInput = document.querySelector('input[name="DepartureDate"]');
    const retInput = document.querySelector('input[name="ReturnDate"]');

    if (!depInput || !retInput) return;

    const data = await loadTripFromOPFS();

    if (data) {
        populateOverview(data.overview || {});
        syncEventsToDateSpan(data.events || []);
    }

    // --- store previous values for reverting ---
    let prevDepartureDate = depInput.value;
    let prevReturnDate = retInput.value;

    function onDateChange() {
        const dep = depInput.value;
        const ret = retInput.value;

        if (!dep || !ret) {
            checkDates(); // show "Select start and end date" if missing
            return;
        }

        const wanted = daysBetween(dep, ret);
        const current = document.querySelectorAll("#eventsFieldset .event").length;

        if (wanted < current) {
            const ok = confirm(`Remove ${current - wanted} last day(s)?`);
            if (!ok) {
                // revert to previous
                depInput.value = prevDepartureDate;
                retInput.value = prevReturnDate;
                return;
            }
        }

        // update previous values
        prevDepartureDate = depInput.value;
        prevReturnDate = retInput.value;

        // rebuild events
        syncEventsToDateSpan(buildEventsArray());

        // update message if either date missing
        checkDates();
    }

    // single listener for both inputs
    depInput.addEventListener("change", onDateChange);
    retInput.addEventListener("change", onDateChange);

    // run validation once on page load
    checkDates();
}
