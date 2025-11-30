SELECT
    *,
    SUBSTR(InnerId, 1, 1) AS TripDomain,
    CAST(strftime('%Y', DepartureDate) AS INTEGER) AS TripYear,
    (
        CAST(strftime('%Y', DepartureDate) AS INTEGER) / 10
    )
    * 10 AS TripDecade
FROM
    bewa_Overview
WHERE
    InnerId IS NOT NULL
ORDER BY
    TripYear DESC,
    DepartureDate ASC;
