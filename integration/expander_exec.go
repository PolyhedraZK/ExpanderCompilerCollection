package integration

import (
	_ "embed"
	"fmt"
	"os"
	"os/exec"
	"runtime"
)

//go:embed bin/expander-exec-linux-avx2
var embed_expander_exec_linux_avx2 []byte

//go:embed bin/expander-exec-macos
var embed_expander_exec_macos []byte

type Prover struct {
	circuitDir string
	filename   string
}

const bin_loc = "./__expander-exec"

// format from external specification, maxConcurrency and releaseFlags are not used now
func NewProver(circuitDir string, filename string, maxConcurrency int, releaseFlag bool) (*Prover, error) {
	switch runtime.GOOS {
	case "darwin":
		err := os.WriteFile(bin_loc, embed_expander_exec_macos, 0700)
		if err != nil {
			panic("could not write file")
		}
	case "linux":
		err := os.WriteFile(bin_loc, embed_expander_exec_linux_avx2, 0700)
		if err != nil {
			panic("could not write file")
		}
	default:
		println("Unsupported OS: ", runtime.GOOS)
		panic("Unsupported OS")
	}
	return &Prover{
		circuitDir: circuitDir,
		filename:   filename,
	}, nil
}

func (p *Prover) Prove(witnessData []byte) ([]byte, error) {
	output_file := "./out.bin"
	// use external prover executable
	// format: ./bin/expander-exec prove <input:circuit_file> <input:witness_file> <output:proof>

	tmpFile, err := os.CreateTemp("", "witness")
	if err != nil {
		fmt.Println("Error creating temporary file:", err)
		return nil, err
	}

	if _, err := tmpFile.Write(witnessData); err != nil {
		fmt.Println("Error writing to temporary file:", err)
		return nil, err
	}

	cmd := exec.Command(bin_loc, "prove", p.circuitDir, tmpFile.Name(), output_file)
	// println("cmd: ", cmd.String())
	if err := cmd.Run(); err != nil {
		fmt.Println("Error running command:", err)
		return nil, err
	}
	defer os.Remove(tmpFile.Name()) // Clean up the file when done

	proof_and_claim, err := os.ReadFile(output_file)
	if err != nil {
		fmt.Println("Error reading output file:", err)
		return nil, err
	}
	os.Remove(output_file)

	return proof_and_claim, nil
}

func (p *Prover) Verify(witnessData []byte, proof_and_claim []byte) (bool, error) {
	// use external prover executable
	// format: ./bin/expander-exec verify <input:circuit_file> <input:witness_file> <input:proof>

	tmpFileWit, err := os.CreateTemp("", "witness")
	if err != nil {
		fmt.Println("Error creating temporary file:", err)
		return false, err
	}

	if _, err := tmpFileWit.Write(witnessData); err != nil {
		fmt.Println("Error writing to temporary file:", err)
		return false, err
	}

	tmpFileProof, err := os.CreateTemp("", "proof")
	if err != nil {
		fmt.Println("Error creating temporary file:", err)
		return false, err
	}

	if _, err := tmpFileProof.Write(proof_and_claim); err != nil {
		fmt.Println("Error writing to temporary file:", err)
		return false, err
	}

	cmd := exec.Command(bin_loc, "verify", p.circuitDir, tmpFileWit.Name(), tmpFileProof.Name())
	// println("cmd: ", cmd.String())
	if err := cmd.Run(); err != nil {
		// no need to print as it is expected to fail on invalid proof
		return false, err
	}
	defer os.Remove(tmpFileWit.Name())   // Clean up the file when done
	defer os.Remove(tmpFileProof.Name()) // Clean up the file when done

	return true, nil
}
