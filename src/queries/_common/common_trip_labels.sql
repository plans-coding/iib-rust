SELECT DISTINCT TRIM(value) AS TripLabel
FROM bewa_Overview,
     json_each('["' || REPLACE(TripLabels, ',', '","') || '"]')
WHERE TripLabels IS NOT NULL AND TripLabel <> ''
ORDER BY TripLabel;
