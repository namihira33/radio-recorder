import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Station {
  id: string;
  name: string;
  area_id: string;
}

interface Program {
  id: string;
  title: string;
  description: string;
  start_time: string;
  end_time: string;
  station_id: string;
  performer: string | null;
}

interface Recording {
  id: string;
  title: string;
  station: string;
  recorded_at: string;
  duration: number;
  file_path: string;
  tags: string[];
}

type View = "library" | "stations" | "schedule" | "settings";
type ServiceType = "nhk" | "radiko";

function App() {
  const [view, setView] = useState<View>("library");
  const [serviceType, setServiceType] = useState<ServiceType>("nhk");
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [stations, setStations] = useState<Station[]>([]);
  const [nhkStations, setNhkStations] = useState<Station[]>([]);
  const [selectedStation, setSelectedStation] = useState<Station | null>(null);
  const [programs, setPrograms] = useState<Program[]>([]);
  const [recordings, setRecordings] = useState<Recording[]>([]);
  const [currentRecording, setCurrentRecording] = useState<Recording | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackProgress, setPlaybackProgress] = useState(0);
  const [playbackSpeed, setPlaybackSpeed] = useState(1);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [audioElement, setAudioElement] = useState<HTMLAudioElement | null>(null);

  useEffect(() => {
    loadLibrary();
    loadNhkStations();
  }, []);

  useEffect(() => {
    if (audioElement) {
      audioElement.playbackRate = playbackSpeed;
    }
  }, [playbackSpeed, audioElement]);

  const loadLibrary = async () => {
    try {
      const library = await invoke<Recording[]>("get_library");
      setRecordings(library);
    } catch (e) {
      console.error("Failed to load library:", e);
    }
  };

  const loadNhkStations = async () => {
    try {
      const stations = await invoke<Station[]>("get_nhk_stations");
      setNhkStations(stations);
    } catch (e) {
      console.error("Failed to load NHK stations:", e);
    }
  };

  const handleLogin = async () => {
    setIsLoading(true);
    setError("");
    try {
      await invoke("authenticate", { email, password });
      setIsAuthenticated(true);
      const stationList = await invoke<Station[]>("get_stations");
      setStations(stationList);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const handleStationSelect = async (station: Station) => {
    setSelectedStation(station);
    setPrograms([]);
    const today = new Date().toISOString().split("T")[0].replace(/-/g, "");

    try {
      if (serviceType === "nhk") {
        const programList = await invoke<Program[]>("get_nhk_programs", {
          stationId: station.id,
          date: today,
        });
        setPrograms(programList);
      } else {
        const programList = await invoke<Program[]>("get_programs", {
          stationId: station.id,
          date: today,
        });
        setPrograms(programList);
      }
    } catch (e) {
      console.error("Failed to load programs:", e);
    }
  };

  const startRecording = async (station: Station, duration: number, title: string) => {
    try {
      const timestamp = new Date().toISOString().split("T")[0];
      const safeName = title.replace(/[/\\?%*:|"<>]/g, "_");

      if (serviceType === "nhk") {
        await invoke("start_nhk_recording", {
          stationId: station.id,
          durationMinutes: duration,
          outputName: `${timestamp}_${safeName}`,
        });
      } else {
        await invoke("start_recording", {
          stationId: station.id,
          durationMinutes: duration,
          outputName: `${timestamp}_${safeName}`,
        });
      }
      // Refresh library after a delay
      setTimeout(loadLibrary, 1000);
    } catch (e) {
      setError(String(e));
    }
  };

  const playRecording = (recording: Recording) => {
    if (audioElement) {
      audioElement.pause();
    }

    const audio = new Audio(`file://${recording.file_path}`);
    audio.playbackRate = playbackSpeed;

    audio.addEventListener("timeupdate", () => {
      setPlaybackProgress((audio.currentTime / audio.duration) * 100);
    });

    audio.addEventListener("ended", () => {
      setIsPlaying(false);
      setPlaybackProgress(0);
    });

    audio.play();
    setAudioElement(audio);
    setCurrentRecording(recording);
    setIsPlaying(true);
  };

  const togglePlayPause = () => {
    if (!audioElement) return;

    if (isPlaying) {
      audioElement.pause();
    } else {
      audioElement.play();
    }
    setIsPlaying(!isPlaying);
  };

  const seek = (percent: number) => {
    if (!audioElement) return;
    audioElement.currentTime = (percent / 100) * audioElement.duration;
  };

  const deleteRecording = async (id: string) => {
    try {
      await invoke("delete_recording", { id });
      loadLibrary();
      if (currentRecording?.id === id) {
        audioElement?.pause();
        setCurrentRecording(null);
        setIsPlaying(false);
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const formatTime = (timeStr: string) => {
    if (timeStr.length === 14) {
      return `${timeStr.slice(8, 10)}:${timeStr.slice(10, 12)}`;
    }
    return timeStr;
  };

  const formatDuration = (minutes: number) => {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return hours > 0 ? `${hours}時間${mins}分` : `${mins}分`;
  };

  const currentStations = serviceType === "nhk" ? nhkStations : stations;

  return (
    <div className="app">
      {/* Sidebar */}
      <aside className="sidebar">
        <div className="logo">
          <h1>Radio Recorder</h1>
        </div>
        <nav>
          <button
            className={view === "library" ? "active" : ""}
            onClick={() => setView("library")}
          >
            ライブラリ
          </button>
          <button
            className={view === "stations" ? "active" : ""}
            onClick={() => setView("stations")}
          >
            放送局
          </button>
          <button
            className={view === "schedule" ? "active" : ""}
            onClick={() => setView("schedule")}
          >
            予約録音
          </button>
          <button
            className={view === "settings" ? "active" : ""}
            onClick={() => setView("settings")}
          >
            設定
          </button>
        </nav>

        {/* Player */}
        {currentRecording && (
          <div className="mini-player">
            <div className="now-playing">
              <strong>{currentRecording.title}</strong>
              <span>{currentRecording.station}</span>
            </div>
            <div className="controls">
              <button onClick={togglePlayPause}>
                {isPlaying ? "⏸" : "▶"}
              </button>
            </div>
            <div
              className="progress-bar"
              onClick={(e) => {
                const rect = e.currentTarget.getBoundingClientRect();
                const percent = ((e.clientX - rect.left) / rect.width) * 100;
                seek(percent);
              }}
            >
              <div className="progress" style={{ width: `${playbackProgress}%` }} />
            </div>
          </div>
        )}
      </aside>

      {/* Main Content */}
      <main className="content">
        {error && <div className="error">{error}</div>}

        {/* Library View */}
        {view === "library" && (
          <div className="library-view">
            <h2>ライブラリ</h2>
            {recordings.length === 0 ? (
              <p className="empty">録音がありません</p>
            ) : (
              <div className="recording-list">
                {recordings.map((recording) => (
                  <div
                    key={recording.id}
                    className={`recording-item ${currentRecording?.id === recording.id ? "active" : ""}`}
                  >
                    <div
                      className="recording-info"
                      onClick={() => playRecording(recording)}
                    >
                      <h3>{recording.title}</h3>
                      <p>
                        {recording.station} • {formatDuration(recording.duration)}
                      </p>
                      <p className="date">
                        {new Date(recording.recorded_at).toLocaleDateString("ja-JP")}
                      </p>
                      {recording.tags.length > 0 && (
                        <div className="tags">
                          {recording.tags.map((tag) => (
                            <span key={tag} className="tag">{tag}</span>
                          ))}
                        </div>
                      )}
                    </div>
                    <button
                      className="delete-btn"
                      onClick={() => deleteRecording(recording.id)}
                    >
                      削除
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Stations View */}
        {view === "stations" && (
          <div className="stations-view">
            <h2>放送局</h2>

            {/* Service Tabs */}
            <div className="service-tabs">
              <button
                className={serviceType === "nhk" ? "active" : ""}
                onClick={() => {
                  setServiceType("nhk");
                  setSelectedStation(null);
                  setPrograms([]);
                }}
              >
                NHK
              </button>
              <button
                className={serviceType === "radiko" ? "active" : ""}
                onClick={() => {
                  setServiceType("radiko");
                  setSelectedStation(null);
                  setPrograms([]);
                }}
              >
                radiko
              </button>
            </div>

            {serviceType === "radiko" && !isAuthenticated ? (
              <div className="login-form">
                <p>radikoにログインしてください</p>
                <input
                  type="email"
                  placeholder="メールアドレス"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                />
                <input
                  type="password"
                  placeholder="パスワード"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
                <button onClick={handleLogin} disabled={isLoading}>
                  {isLoading ? "ログイン中..." : "ログイン"}
                </button>
                <p className="hint">
                  ※ 無料エリアのみの場合は空欄でもOK
                </p>
              </div>
            ) : (
              <div className="station-content">
                <div className="station-list">
                  {currentStations.map((station) => (
                    <button
                      key={station.id}
                      className={selectedStation?.id === station.id ? "active" : ""}
                      onClick={() => handleStationSelect(station)}
                    >
                      {station.name}
                    </button>
                  ))}
                </div>
                {selectedStation && (
                  <div className="program-list">
                    <h3>{selectedStation.name} - 番組表</h3>
                    {programs.length === 0 ? (
                      <p className="empty">番組情報を取得中...</p>
                    ) : (
                      programs.map((program) => (
                        <div key={program.id} className="program-item">
                          <div className="program-time">
                            {formatTime(program.start_time)} - {formatTime(program.end_time)}
                          </div>
                          <div className="program-info">
                            <h4>{program.title}</h4>
                            {program.performer && <p>{program.performer}</p>}
                          </div>
                          <button
                            className="record-btn"
                            onClick={() => {
                              const start = new Date(
                                `${program.start_time.slice(0, 4)}-${program.start_time.slice(4, 6)}-${program.start_time.slice(6, 8)}T${program.start_time.slice(8, 10)}:${program.start_time.slice(10, 12)}:00`
                              );
                              const end = new Date(
                                `${program.end_time.slice(0, 4)}-${program.end_time.slice(4, 6)}-${program.end_time.slice(6, 8)}T${program.end_time.slice(8, 10)}:${program.end_time.slice(10, 12)}:00`
                              );
                              const duration = Math.ceil((end.getTime() - start.getTime()) / 60000);
                              startRecording(selectedStation, duration, program.title);
                            }}
                          >
                            録音
                          </button>
                        </div>
                      ))
                    )}
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {/* Schedule View */}
        {view === "schedule" && (
          <div className="schedule-view">
            <h2>予約録音</h2>
            <p className="coming-soon">この機能は準備中です</p>
          </div>
        )}

        {/* Settings View */}
        {view === "settings" && (
          <div className="settings-view">
            <h2>設定</h2>
            <div className="setting-item">
              <label>再生速度</label>
              <select
                value={playbackSpeed}
                onChange={(e) => setPlaybackSpeed(Number(e.target.value))}
              >
                <option value={0.5}>0.5x</option>
                <option value={0.75}>0.75x</option>
                <option value={1}>1x</option>
                <option value={1.25}>1.25x</option>
                <option value={1.5}>1.5x</option>
                <option value={2}>2x</option>
              </select>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
