SELECT
    *,
    CAST(julianday(ReturnDate) - julianday(DepartureDate) AS INTEGER) AS NumberOfDays,
    TripDomain AS DomainAbbreviation
FROM
    bewa_Overview
WHERE
    OuterId = '/*_OUTER_ID_*/'
LIMIT 1;
