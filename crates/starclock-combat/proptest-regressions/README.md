# Proptest regression corpus

The fixed ChaCha seeds in the combat property suites make every run
reproducible. Proptest writes minimized failures below this directory through
`FileFailurePersistence::SourceParallel`. Commit any generated regression file
unchanged; it becomes a permanent corpus input on later runs.
