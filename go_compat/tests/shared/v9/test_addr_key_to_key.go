package main

import (
	"encoding/hex"
	"flag"
	"fmt"

	"github.com/filecoin-project/go-address"
	"github.com/filecoin-project/go-state-types/abi"
)

func main() {
	var addr string

	flag.StringVar(&addr, "addr", "", "address bytes in hex")

	flag.Parse()

	addrBytes, err := hex.DecodeString(addr)
	if err != nil {
		panic(err)
	}

	a, err := address.NewFromBytes(addrBytes)
	if err != nil {
		panic(err)
	}

	key := abi.IdAddrKey(a)

	fmt.Print(hex.EncodeToString([]byte(key.Key())))
}
