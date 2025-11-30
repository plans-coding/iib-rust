SELECT
    AccommodationCountry, COUNT(*) AS Overnights
FROM
    bewb_Events
GROUP BY
    AccommodationCountry
ORDER BY
    Overnights DESC;
