package integration

import (
	_ "embed"
	"fmt"
	"os"
	"os/exec"
	"runtime"

	"github.com/juju/fslock"
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
const bin_lock_loc = "./__expander-exec.lock"

// format from external specification, maxConcurrency and releaseFlags are not used now
func NewProver(circuitDir string, filename string, maxConcurrency int, releaseFlag bool) (*Prover, error) {
	lock := fslock.New(bin_lock_loc)
	lock.Lock()
	if _, err := os.Stat(bin_loc); os.IsNotExist(err) {
		// write the binary to the file
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
	}
	lock.Unlock()
	return &Prover{
		circuitDir: circuitDir,
		filename:   filename,
	}, nil
}

func (p *Prover) Prove(witnessData []byte) ([]byte, error) {
	// use external prover executable
	// format: ./bin/expander-exec prove <input:circuit_file> <input:witness_file> <output:proof>
	outputFile, err := os.CreateTemp("", "output")
	if err != nil {
		fmt.Println("Error creating temporary file:", err)
		return nil, err
	}

	tmpFile, err := os.CreateTemp("", "witness")
	if err != nil {
		fmt.Println("Error creating temporary file:", err)
		return nil, err
	}

	if _, err := tmpFile.Write(witnessData); err != nil {
		fmt.Println("Error writing to temporary file:", err)
		return nil, err
	}

	cmd := exec.Command(bin_loc, "prove", p.circuitDir, tmpFile.Name(), outputFile.Name())
	// println("cmd: ", cmd.String())
	if err := cmd.Run(); err != nil {
		fmt.Println("Error running command:", err)
		return nil, err
	}
	defer os.Remove(tmpFile.Name()) // Clean up the file when done

	proof_and_claim, err := os.ReadFile(outputFile.Name())
	if err != nil {
		fmt.Println("Error reading output file:", err)
		return nil, err
	}
	os.Remove(outputFile.Name())
	defer os.Remove(outputFile.Name()) // Clean up the file when done

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
	println("cmd: ", cmd.String())
	if err := cmd.Run(); err != nil {
		// no need to print as it is expected to fail on invalid proof
		return false, err
	}
	defer os.Remove(tmpFileWit.Name())   // Clean up the file when done
	defer os.Remove(tmpFileProof.Name()) // Clean up the file when done

	return true, nil
}
