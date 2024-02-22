#! /bin/bash
 
runtime="$RUNTIME"
export CONFIG="/app/config.yaml"

if [ "$runtime" == "spiritnet" ]; then
        /app/attester_spiritnet 
fi

if [ "$runtime" == "peregrine" ]; then
        /app/attester_peregrine
fi

echo "no valid runtime provided"
exit 1
