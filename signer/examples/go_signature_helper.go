package main

import (
	"encoding/hex"
	"fmt"
	"os"
	
	curve "github.com/elliottech/poseidon_crypto/curve/ecgfp5"
	gFp5 "github.com/elliottech/poseidon_crypto/field/goldilocks_quintic_extension"
	schnorr "github.com/elliottech/poseidon_crypto/signature/schnorr"
	p2 "github.com/elliottech/poseidon_crypto/hash/poseidon2_goldilocks"
)

// This helper generates a signature with a fixed nonce for comparison with Rust
// Usage: go run go_signature_helper.go <private_key_hex> <message_hex> <nonce_hex>
func main() {
	if len(os.Args) != 4 {
		fmt.Fprintf(os.Stderr, "Usage: %s <private_key_hex> <message_hex> <nonce_hex>\n", os.Args[0])
		os.Exit(1)
	}
	
	privateKeyHex := os.Args[1]
	messageHex := os.Args[2]
	nonceHex := os.Args[3]
	
	// Parse hex strings
	privateKeyBytes, err := hex.DecodeString(privateKeyHex)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error decoding private key: %v\n", err)
		os.Exit(1)
	}
	
	messageBytes, err := hex.DecodeString(messageHex)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error decoding message: %v\n", err)
		os.Exit(1)
	}
	
	nonceBytes, err := hex.DecodeString(nonceHex)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error decoding nonce: %v\n", err)
		os.Exit(1)
	}
	
	// Convert to types
	privateKey := curve.ScalarElementFromLittleEndianBytes(privateKeyBytes)
	messageFp5, err := gFp5.FromCanonicalLittleEndianBytes(messageBytes)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error converting message: %v\n", err)
		os.Exit(1)
	}
	
	nonceScalar := curve.ScalarElementFromLittleEndianBytes(nonceBytes)
	
	// Compute R = nonce * G
	generator := curve.GENERATOR_ECgFp5Point
	rPoint := generator.Mul(&nonceScalar)
	rEncoded := rPoint.Encode()
	
	// Compute e = H(R || message) using Poseidon2
	var preImage [10]gFp5.Element
	for i := 0; i < 5; i++ {
		preImage[i] = rEncoded[i]
	}
	for i := 0; i < 5; i++ {
		preImage[5+i] = messageFp5[i]
	}
	
	poseidon2 := p2.NewPoseidon2()
	eFp5 := poseidon2.HashToQuinticExtension(preImage[:])
	eScalar := curve.ScalarFromFp5Element(&eFp5)
	
	// Compute s = nonce - e * private_key
	eTimesPrivate := eScalar.Mul(&privateKey)
	s := nonceScalar.Sub(eTimesPrivate)
	
	// Generate signature (for comparison, we'll also use the actual signing function)
	// Note: Go's SchnorrSignHashedMessage generates its own nonce, so we need to
	// manually construct the signature with our fixed nonce
	signature := make([]byte, 80)
	copy(signature[:40], s.ToLittleEndianBytes())
	copy(signature[40:], eScalar.ToLittleEndianBytes())
	
	// Output results in a parseable format
	fmt.Printf("R_ENCODED:")
	for _, elem := range rEncoded {
		fmt.Printf(" %d", elem)
	}
	fmt.Println()
	
	fmt.Printf("E_FP5:")
	for i := 0; i < 5; i++ {
		fmt.Printf(" %d", eFp5[i])
	}
	fmt.Println()
	
	fmt.Printf("E_SCALAR:%s\n", hex.EncodeToString(eScalar.ToLittleEndianBytes()))
	fmt.Printf("S_SCALAR:%s\n", hex.EncodeToString(s.ToLittleEndianBytes()))
	fmt.Printf("SIGNATURE:%s\n", hex.EncodeToString(signature))
}

