SELECT
    DISTINCT AccommodationCountry
FROM
    IIBb_Events
WHERE
    AccommodationCountry NOT LIKE '(%'
    AND AccommodationCountry NOT LIKE '-%'
ORDER BY
    AccommodationCountry;