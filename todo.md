please add sqllite caching with ORM for fetching prices. A->B is same cost as B->A, prices don't change until the first on january, so that'd be the date to fetch new data (ie expiration field in db would be nice). The lookup key would be normalized(A,B,travel_class) that fetches a price or None (if not found or expired). after fetching the new record, the db should be updated. 

html frontend? 

later:

Add CI/CD github actions to build executables for all major OS's, so people can run it themselves. the on release schedule that I have is nice I think! For the server it would also be nice to pro