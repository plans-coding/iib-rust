WITH MovieLines AS (
    SELECT
        trim(j.value) AS line
    FROM bewx_Settings AS s,
         json_each('["' || replace(s.Value, char(10), '","') || '"]') AS j
    WHERE s.Attribute = 'Movie'
)
SELECT
    substr(line, instr(line, ':') + 1) AS MovieId
FROM MovieLines
WHERE line LIKE '_OUTER_ID_:%';
