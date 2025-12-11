SELECT
    InnerId,
    GROUP_CONCAT(AccommodationCoordinates, '|') AS MergedAccommodationCoordinates
FROM
    bewb_Events
WHERE
    AccommodationCoordinates IS NOT NULL
GROUP BY
    InnerId;