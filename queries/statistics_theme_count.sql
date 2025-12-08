SELECT
    json_extract(j.value, '$.key') AS key,
    SUM(
        (LENGTH(e.AdditionalNotes) - LENGTH(REPLACE(e.AdditionalNotes, '| ' || json_extract(j.value, '$.key'), '')))
        / LENGTH('| ' || json_extract(j.value, '$.key'))
    ) AS total_count
FROM
    bewb_Events AS e
CROSS JOIN
    (SELECT j.value
    FROM bewxx_Settings AS s,
            json_each(s.Value, '$.mapping') AS j
    WHERE s.Attribute = 'Theme') AS j
WHERE
    e.AdditionalNotes LIKE '%| ' || json_extract(j.value, '$.key') || '%'
GROUP BY
    key;
