package main

import (
	"bytes"
	"encoding/hex"
	"flag"
	"fmt"
	"strings"

	commp "github.com/filecoin-project/go-commp-utils/nonffi"
	"github.com/filecoin-project/go-state-types/abi"
)

const FIL_COMMITMENT_UNSEALED = 0xf101

func main() {
	var proof int64
	var piecesHex string

	flag.Int64Var(&proof, "proof", 0, "proof type")
	flag.StringVar(&piecesHex, "pieces", "", "piece hex list delimited by comma")

	flag.Parse()

	proofType := abi.RegisteredSealProof(proof)

	pieces := make([]abi.PieceInfo, 0)
	pieceHexList := strings.Split(piecesHex, ",")
	for i := 0; i < len(pieceHexList); i++ {
		pieceBytes, err := hex.DecodeString(pieceHexList[i])
		if err != nil {
			panic(err)
		}
		buf := new(bytes.Buffer)
		buf.Write(pieceBytes)
		var piece abi.PieceInfo
		if err := piece.UnmarshalCBOR(buf); err != nil {
			panic(err)
		}

		pieces = append(pieces, piece)
	}

	cid, err := commp.GenerateUnsealedCID(proofType, pieces)
	if err != nil {
		panic(err)
	}
	fmt.Print(cid)
}
