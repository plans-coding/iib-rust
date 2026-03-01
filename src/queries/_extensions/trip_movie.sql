WITH SplitLines AS (
    SELECT trim(j.value) AS line
    FROM bewx_Settings AS s,
         json_each('["' || replace(s.Value, char(10), '","') || '"]') AS j
    WHERE s.Attribute = 'Movie'
),
FirstRow AS (
    SELECT line FROM SplitLines
    LIMIT 1
),
UrlPrefix AS (
    SELECT trim(j2.value) AS prefix
    FROM FirstRow,
         json_each('["' || replace(line, ',', '","') || '"]') AS j2
    WHERE j2.key = 2  -- third element (0-based)
)
SELECT
    substr(line, instr(line, ':') + 1) AS MovieId,
    UrlPrefix.prefix || substr(line, instr(line, ':') + 1) AS MovieUrl
FROM SplitLines, UrlPrefix
WHERE line LIKE '_OUTER_ID_:%';