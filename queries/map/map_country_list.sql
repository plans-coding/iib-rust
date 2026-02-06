SELECT
    DISTINCT AccommodationCountry
FROM
    bewb_Events
WHERE
    AccommodationCountry NOT LIKE '(%'
    AND AccommodationCountry NOT LIKE '-%'
ORDER BY
    AccommodationCountry;
