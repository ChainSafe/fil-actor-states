package main

import (
	"encoding/hex"
	"flag"
	"fmt"

	"github.com/filecoin-project/go-state-types/abi"
	miner9 "github.com/filecoin-project/go-state-types/builtin/v9/miner"
)

func main() {
	var sector uint64

	flag.Uint64Var(&sector, "sector", 0, "sector number")

	flag.Parse()

	key := miner9.SectorKey(abi.SectorNumber(sector))

	fmt.Print(hex.EncodeToString([]byte(key.Key())))
}
