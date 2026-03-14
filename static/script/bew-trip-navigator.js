function init_trip_navigator() {
    const sidebar = document.getElementById('trip-side-navigator');
    const resizer = document.getElementById('trip-side-navigator-resizer');

    let startX;
    let startWidth;

    resizer.addEventListener('mousedown', e => {
        e.preventDefault();          // ← prevent text selection / drag ghosts
        startX = e.clientX;
        startWidth = sidebar.getBoundingClientRect().width;

        document.body.style.cursor = 'ew-resize';
        document.body.style.userSelect = 'none';

        document.addEventListener('mousemove', resize);
        document.addEventListener('mouseup', stopResize);
    });

    function resize(e) {
        const delta = e.clientX - startX;
        const width = Math.max(150, Math.min(600, startWidth + delta));
        sidebar.style.width = width + 'px';
    }

    function stopResize() {
        document.body.style.cursor = '';
        document.body.style.userSelect = '';

        document.removeEventListener('mousemove', resize);
        document.removeEventListener('mouseup', stopResize);
    }
}

async function toggle_trip_side_navigator() {

    document.getElementById('trip-side-navigator').style.display
    = (document.getElementById('trip-side-navigator').style.display === 'none' ? 'block' : 'none');

    if (document.getElementById('trip-side-navigator').style.display == "block") {
        await page_load('trip-side-navigator-content', true);
        replace_links();
    }
}

async function load_trip_side_navigator(destination, choice) {
    await page_load(destination, choice);
    replace_links();
}

function replace_links() {
    document
    .querySelectorAll(`#trip-side-navigator a[href^="?path=overview:year"]`)
    .forEach(link => {
        link.removeAttribute('href');
        link.setAttribute(
            "onclick",
            `load_trip_side_navigator('trip-side-navigator-content', 1);`
        );
    });

    document
    .querySelectorAll(`#trip-side-navigator a[href^="?path=overview:country"]`)
    .forEach(link => {
        link.removeAttribute('href');
        link.setAttribute(
            "onclick",
            `load_trip_side_navigator('trip-side-navigator-content', 2);`
        );
    });
}
