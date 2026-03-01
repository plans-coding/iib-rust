-- This code takes DepartureDate and Return date from bewa_Overview. bewb_Events can contain a broader span that will not be catch below
WITH RECURSIVE

/* --------------------------------------------------
   1. Get overview date span
-------------------------------------------------- */
overview AS (
    SELECT
        DepartureDate,
        ReturnDate
    FROM bewa_Overview
    WHERE OuterId = '_OUTER_ID_'
),

/* --------------------------------------------------
   2. Expand each day in the span
-------------------------------------------------- */
dates(d) AS (
    SELECT date(DepartureDate) FROM overview
    UNION ALL
    SELECT date(d, '+1 day')
    FROM dates, overview
    WHERE d < ReturnDate
),

/* --------------------------------------------------
   3. Split BOTH Diary + Passport blobs into lines
-------------------------------------------------- */
split(attr, line, rest) AS (
    SELECT
        Attribute,
        '',
        Value || char(10)
    FROM bewx_Settings
    WHERE Attribute IN ('Diary','Passport')

    UNION ALL

    SELECT
        attr,
        substr(rest, 0, instr(rest, char(10))),
        substr(rest, instr(rest, char(10)) + 1)
    FROM split
    WHERE rest <> ''
),

rows AS (
    SELECT
        attr,
        line,
        row_number() OVER (PARTITION BY attr) AS rn
    FROM split
    WHERE line <> ''
),

/* --------------------------------------------------
   4. Config row (first line)
-------------------------------------------------- */
config AS (
    SELECT
        attr,
        substr(line, 1, instr(line, ',')-1) AS enabled,
        substr(
            line,
            instr(line, ',') + instr(substr(line, instr(line, ',')+1), ',') + 1
        ) AS base_url
    FROM rows
    WHERE rn = 1
),

/* --------------------------------------------------
   5. File rows
-------------------------------------------------- */
files AS (
    SELECT
        attr,
        line AS file
    FROM rows
    WHERE rn > 1
)

/* --------------------------------------------------
   6. Final unified result
-------------------------------------------------- */
SELECT
    d.d                       AS Date,
	f.attr                    AS ExtensionType,
    c.base_url || f.file      AS Filepath
FROM files f
JOIN config c   ON c.attr = f.attr
JOIN dates  d
WHERE
    c.enabled = 'Enabled'
    AND (
        /* Diary rule ISO year: %G ISO week: %V */
        (f.attr = 'Diary'
         AND f.file LIKE '%' || strftime('%G', d.d) || '%'
         AND f.file LIKE '%W' || strftime('%V', d.d) || '%')

        OR

        /* Passport rule */
        (f.attr = 'Passport'
         AND f.file LIKE '%' || strftime('%Y%m%d', d.d) || '%')
    )
ORDER BY d.d, f.attr;
