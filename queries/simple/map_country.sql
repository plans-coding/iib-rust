SELECT
    * FROM IIBb_Events
WHERE
    AccommodationCountry = "${parameter}"
    AND AccommodationCoordinates IS NOT NULL;