package integration

import (
	"bytes"
	_ "embed"
	"encoding/binary"
	"fmt"
	"io"
	"net/http"
	"net/url"
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

func is_url(s string) bool {
	_, err := url.ParseRequestURI(s)
	return err == nil
}

func GetIntegrationBinLoc() string {
	switch runtime.GOOS {
	case "darwin":
		return "./integration/bin/expander-exec-macos"
	case "linux":
		return "./integration/bin/expander-exec-linux-avx2"
	default:
		println("Unsupported OS: ", runtime.GOOS)
		panic("Unsupported OS")
	}
}

// format from external specification, maxConcurrency and releaseFlags are not used now
func NewProver(circuitDir string, filename string, maxConcurrency int, releaseFlag bool) (*Prover, error) {
	if is_url(filename) {
		return &Prover{
			circuitDir: circuitDir,
			filename:   filename,
		}, nil
	}
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
	if is_url(p.filename) {
		proveReq, err := http.NewRequest("POST", p.filename+"/prove", bytes.NewBuffer(witnessData))
		if err != nil {
			return nil, err
		}
		proveReq.Header.Set("Content-Type", "application/octet-stream")
		proveReq.Header.Set("Content-Length", fmt.Sprintf("%d", len(witnessData)))
		client := &http.Client{}
		proveResp, err := client.Do(proveReq)
		if err != nil {
			return nil, err
		}
		defer proveResp.Body.Close()

		proof_and_claim, err := io.ReadAll(proveResp.Body)
		if err != nil {
			return nil, err
		}
		return proof_and_claim, nil
	}
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
	if is_url(p.filename) {
		witnessLen := make([]byte, 8)
		binary.LittleEndian.PutUint64(witnessLen, uint64(len(witnessData)))
		proofLen := make([]byte, 8)
		binary.LittleEndian.PutUint64(proofLen, uint64(len(proof_and_claim)))
		verifierInput := append(append(append(witnessLen, proofLen...), witnessData...), proof_and_claim...)
		verifyReq, err := http.NewRequest("POST", p.filename+"/verify", bytes.NewBuffer(verifierInput))
		if err != nil {
			return false, err
		}
		verifyReq.Header.Set("Content-Type", "application/octet-stream")
		verifyReq.Header.Set("Content-Length", fmt.Sprintf("%d", len(verifierInput)))
		client := &http.Client{}
		verifyResp, err := client.Do(verifyReq)
		if err != nil {
			return false, err
		}
		defer verifyResp.Body.Close()

		verificationResult, err := io.ReadAll(verifyResp.Body)
		if err != nil {
			return false, err
		}
		return string(verificationResult) == "success", nil
	}
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
