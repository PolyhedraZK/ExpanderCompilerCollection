package wrapper

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"github.com/consensys/gnark/logger"
)

func getCacheDir() (string, error) {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	cacheDir := filepath.Join(homeDir, ".cache", "ExpanderCompilerCollection")
	err = os.MkdirAll(cacheDir, 0755)
	return cacheDir, err
}

func downloadFile(url string, filepath string) error {
	out, err := os.Create(filepath)
	if err != nil {
		return err
	}
	defer out.Close()

	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return err
	}

	return nil
}

func getUrl(url string) ([]byte, error) {
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("bad status: %s", resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	return body, nil
}

func downloadLib(path string) {
	log := logger.Logger()
	log.Info().Msg("Downloading rust libs ...")
	err := downloadFile("https://github.com/PolyhedraZK/ExpanderCompilerCollection/raw/rust-built-libs/"+getLibName(), path)
	if err != nil {
		os.Remove(path)
		panic(err)
	}
}

type repoInfo struct {
	Commit struct {
		Commit struct {
			Committer struct {
				Date string `json:"date"`
			} `json:"committer"`
		} `json:"commit"`
	} `json:"commit"`
}

func updateLib(path string) {
	stat, err := os.Stat(path)
	fileExists := !os.IsNotExist(err)
	if err != nil && fileExists {
		panic(err)
	}
	data, err := getUrl("https://api.github.com/repos/PolyhedraZK/ExpanderCompilerCollection/branches/rust-built-libs")
	if err != nil {
		if fileExists {
			return
		}
		panic(err)
	}
	var repoInfo repoInfo
	err = json.Unmarshal(data, &repoInfo)
	if err != nil {
		if fileExists {
			return
		}
		panic(err)
	}
	remoteTime, err := time.Parse(time.RFC3339, repoInfo.Commit.Commit.Committer.Date)
	if err != nil {
		if fileExists {
			return
		}
		panic(err)
	}
	if fileExists {
		localTime := stat.ModTime()
		if localTime.After(remoteTime) {
			return
		}
	}
	downloadLib(path)
}
