SELECT OuterId
FROM bewa_Overview
WHERE OuterID IS NOT NULL AND DepartureDate > (
    SELECT DepartureDate
    FROM bewa_Overview
    WHERE OuterId = '/*_OUTER_ID_*/'
)
ORDER BY DepartureDate ASC
LIMIT 1;