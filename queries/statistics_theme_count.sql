SELECT
    kv.key   AS ThemeAbbreviation,
    kv.value AS ThemeDescription,
    SUM(
        (LENGTH(e.AdditionalNotes)
         - LENGTH(REPLACE(e.AdditionalNotes, '| ' || kv.key, '')))
        / LENGTH('| ' || kv.key)
    ) AS ThemeCount
FROM
    bewb_Events AS e
CROSS JOIN
    (
        SELECT
            kv.key,
            kv.value
        FROM
            bewxx_Settings AS s
        JOIN
            json_each(s.Value, '$.mapping') AS j
        JOIN
            json_each(j.value) AS kv
        WHERE
            s.Attribute = 'Theme'
    ) AS kv
WHERE
    e.AdditionalNotes LIKE '%| ' || kv.key || '%'
GROUP BY
    kv.key,
    kv.value
ORDER BY
	ThemeCount DESC;