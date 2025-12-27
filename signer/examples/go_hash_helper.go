package main

import (
	"encoding/hex"
	"fmt"
	"os"
	"strconv"
	"strings"
	
	curve "github.com/elliottech/poseidon_crypto/curve/ecgfp5"
	g "github.com/elliottech/poseidon_crypto/field/goldilocks"
	p2 "github.com/elliottech/poseidon_crypto/hash/poseidon2_goldilocks"
)

// This helper computes hash values for comparison with Rust
// Usage: go run go_hash_helper.go <r_limbs> <message_limbs>
//   r_limbs: 5 Goldilocks values separated by commas (e.g., "123,456,789,101,112")
//   message_limbs: 5 Goldilocks values separated by commas
func main() {
	if len(os.Args) != 3 {
		fmt.Fprintf(os.Stderr, "Usage: %s <r_limbs> <message_limbs>\n", os.Args[0])
		fmt.Fprintf(os.Stderr, "  r_limbs: 5 Goldilocks values separated by commas\n")
		fmt.Fprintf(os.Stderr, "  message_limbs: 5 Goldilocks values separated by commas\n")
		fmt.Fprintf(os.Stderr, "Example: %s \"123,456,789,101,112\" \"213,314,415,516,617\"\n", os.Args[0])
		os.Exit(1)
	}
	
	// Parse R limbs
	rLimbsStr := strings.Split(os.Args[1], ",")
	if len(rLimbsStr) != 5 {
		fmt.Fprintf(os.Stderr, "Error: R must have exactly 5 limbs, got %d\n", len(rLimbsStr))
		os.Exit(1)
	}
	
	// Parse R limbs as Goldilocks elements (not Fp5Element!)
	var rEncoded [5]g.Element
	for i, limbStr := range rLimbsStr {
		limbVal, err := strconv.ParseUint(strings.TrimSpace(limbStr), 10, 64)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error parsing R limb %d: %v\n", i, err)
			os.Exit(1)
		}
		rEncoded[i] = g.FromUint64(limbVal)
	}

	// Parse message limbs as Goldilocks elements
	messageLimbsStr := strings.Split(os.Args[2], ",")
	if len(messageLimbsStr) != 5 {
		fmt.Fprintf(os.Stderr, "Error: Message must have exactly 5 limbs, got %d\n", len(messageLimbsStr))
		os.Exit(1)
	}

	var messageElems [5]g.Element
	for i, limbStr := range messageLimbsStr {
		limbVal, err := strconv.ParseUint(strings.TrimSpace(limbStr), 10, 64)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error parsing message limb %d: %v\n", i, err)
			os.Exit(1)
		}
		messageElems[i] = g.FromUint64(limbVal)
	}

	// Construct pre-image: [R[0..5], message[0..5]] as Goldilocks elements
	preImage := make([]g.Element, 0, 10)
	for i := 0; i < 5; i++ {
		preImage = append(preImage, rEncoded[i])
	}
	for i := 0; i < 5; i++ {
		preImage = append(preImage, messageElems[i])
	}

	// Compute hash using Poseidon2 (package-level function expects []g.Element)
	eFp5 := p2.HashToQuinticExtension(preImage)
	eScalar := curve.ScalarFromFp5Element(&eFp5)
	
	// Output results
	fmt.Println("=== Go Hash Computation ===")
	fmt.Println("Pre-image (10 Goldilocks elements):")
	for i := 0; i < 10; i++ {
		if i < 5 {
			fmt.Printf("  [%d] R[%d] = %d\n", i, i, preImage[i])
		} else {
			fmt.Printf("  [%d] M[%d] = %d\n", i, i-5, preImage[i])
		}
	}
	
	fmt.Println("\nHash result (Fp5Element):")
	for i := 0; i < 5; i++ {
		fmt.Printf("  e[%d] = %d\n", i, eFp5[i])
	}
	
	fmt.Printf("\nHash result (Scalar): %s\n", hex.EncodeToString(eScalar.ToLittleEndianBytes()))
}


