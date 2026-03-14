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
