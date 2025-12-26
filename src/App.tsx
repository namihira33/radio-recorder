import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Radio,
  Library,
  Calendar,
  Settings,
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Trash2,
  Mic,
  Volume2,
  Loader2,
  AlertCircle,
  X,
} from "lucide-react";
import { cn } from "./lib/utils";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "./components/ui/card";
import { Input } from "./components/ui/input";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./components/ui/tabs";
import { Slider } from "./components/ui/slider";
import { Select } from "./components/ui/select";
import { Badge } from "./components/ui/badge";
import { ScrollArea } from "./components/ui/scroll-area";

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
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [playbackSpeed, setPlaybackSpeed] = useState(1);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isProgramLoading, setIsProgramLoading] = useState(false);
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
    setIsProgramLoading(true);
    setError("");
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
      setError("番組情報の取得に失敗しました: " + String(e));
    } finally {
      setIsProgramLoading(false);
    }
  };

  const startRecording = async (station: Station, durationMinutes: number, title: string) => {
    try {
      const timestamp = new Date().toISOString().split("T")[0];
      const safeName = title.replace(/[/\\?%*:|"<>]/g, "_");

      if (serviceType === "nhk") {
        await invoke("start_nhk_recording", {
          stationId: station.id,
          durationMinutes: durationMinutes,
          outputName: `${timestamp}_${safeName}`,
        });
      } else {
        await invoke("start_recording", {
          stationId: station.id,
          durationMinutes: durationMinutes,
          outputName: `${timestamp}_${safeName}`,
        });
      }
      setTimeout(loadLibrary, 1000);
    } catch (e) {
      setError(String(e));
    }
  };

  const playRecording = useCallback((recording: Recording) => {
    if (audioElement) {
      audioElement.pause();
    }

    const audio = new Audio(`file://${recording.file_path}`);
    audio.playbackRate = playbackSpeed;

    audio.addEventListener("loadedmetadata", () => {
      setDuration(audio.duration);
    });

    audio.addEventListener("timeupdate", () => {
      setCurrentTime(audio.currentTime);
      setPlaybackProgress((audio.currentTime / audio.duration) * 100);
    });

    audio.addEventListener("ended", () => {
      setIsPlaying(false);
      setPlaybackProgress(0);
      setCurrentTime(0);
    });

    audio.play();
    setAudioElement(audio);
    setCurrentRecording(recording);
    setIsPlaying(true);
  }, [audioElement, playbackSpeed]);

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

  const skipForward = () => {
    if (!audioElement) return;
    audioElement.currentTime = Math.min(audioElement.currentTime + 15, audioElement.duration);
  };

  const skipBack = () => {
    if (!audioElement) return;
    audioElement.currentTime = Math.max(audioElement.currentTime - 15, 0);
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

  const formatSeconds = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const navItems = [
    { id: "library" as View, label: "ライブラリ", icon: Library },
    { id: "stations" as View, label: "放送局", icon: Radio },
    { id: "schedule" as View, label: "予約録音", icon: Calendar },
    { id: "settings" as View, label: "設定", icon: Settings },
  ];

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <aside className="w-64 border-r border-border flex flex-col">
        {/* Logo */}
        <div className="p-6 border-b border-border">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
              <Mic className="w-4 h-4 text-primary-foreground" />
            </div>
            <span className="font-semibold text-lg">Radio Recorder</span>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-4">
          <div className="space-y-1">
            {navItems.map((item) => (
              <Button
                key={item.id}
                variant={view === item.id ? "secondary" : "ghost"}
                className={cn(
                  "w-full justify-start gap-3",
                  view === item.id && "bg-primary text-primary-foreground hover:bg-primary/90"
                )}
                onClick={() => setView(item.id)}
              >
                <item.icon className="h-4 w-4" />
                {item.label}
              </Button>
            ))}
          </div>
        </nav>

        {/* Mini Player */}
        {currentRecording && (
          <div className="p-4 border-t border-border bg-card/50">
            <div className="space-y-3">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                  <Volume2 className="w-4 h-4 text-primary" />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium truncate">{currentRecording.title}</p>
                  <p className="text-xs text-muted-foreground truncate">{currentRecording.station}</p>
                </div>
              </div>

              <div className="space-y-2">
                <Slider
                  value={playbackProgress}
                  onChange={(value) => seek(value)}
                  className="cursor-pointer"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>{formatSeconds(currentTime)}</span>
                  <span>{formatSeconds(duration)}</span>
                </div>
              </div>

              <div className="flex items-center justify-center gap-2">
                <Button variant="ghost" size="icon" onClick={skipBack}>
                  <SkipBack className="h-4 w-4" />
                </Button>
                <Button
                  variant="default"
                  size="icon"
                  className="h-10 w-10"
                  onClick={togglePlayPause}
                >
                  {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4 ml-0.5" />}
                </Button>
                <Button variant="ghost" size="icon" onClick={skipForward}>
                  <SkipForward className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>
        )}
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        <ScrollArea className="h-full">
          <div className="p-8">
            {error && (
              <div className="mb-6 p-4 rounded-lg bg-destructive/10 border border-destructive/20 flex items-center gap-3">
                <AlertCircle className="h-5 w-5 text-destructive flex-shrink-0" />
                <p className="text-sm text-destructive flex-1">{error}</p>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() => setError("")}
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
            )}

            {/* Library View */}
            {view === "library" && (
              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <h1 className="text-2xl font-bold">ライブラリ</h1>
                  <Badge variant="secondary">{recordings.length} 件</Badge>
                </div>

                {recordings.length === 0 ? (
                  <Card>
                    <CardContent className="flex flex-col items-center justify-center py-16">
                      <Library className="h-12 w-12 text-muted-foreground/50 mb-4" />
                      <p className="text-muted-foreground">録音がありません</p>
                      <p className="text-sm text-muted-foreground mt-1">
                        放送局から番組を録音してください
                      </p>
                    </CardContent>
                  </Card>
                ) : (
                  <div className="grid gap-4">
                    {recordings.map((recording) => (
                      <Card
                        key={recording.id}
                        className={cn(
                          "transition-all hover:shadow-md cursor-pointer",
                          currentRecording?.id === recording.id && "ring-2 ring-primary"
                        )}
                        onClick={() => playRecording(recording)}
                      >
                        <CardContent className="p-4">
                          <div className="flex items-center gap-4">
                            <div className="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                              {currentRecording?.id === recording.id && isPlaying ? (
                                <Pause className="h-5 w-5 text-primary" />
                              ) : (
                                <Play className="h-5 w-5 text-primary" />
                              )}
                            </div>
                            <div className="flex-1 min-w-0">
                              <h3 className="font-medium truncate">{recording.title}</h3>
                              <div className="flex items-center gap-2 mt-1">
                                <span className="text-sm text-muted-foreground">{recording.station}</span>
                                <span className="text-muted-foreground">·</span>
                                <span className="text-sm text-muted-foreground">
                                  {formatDuration(recording.duration)}
                                </span>
                              </div>
                              <p className="text-xs text-muted-foreground mt-1">
                                {new Date(recording.recorded_at).toLocaleDateString("ja-JP", {
                                  year: "numeric",
                                  month: "long",
                                  day: "numeric",
                                })}
                              </p>
                              {recording.tags.length > 0 && (
                                <div className="flex gap-1.5 mt-2">
                                  {recording.tags.map((tag) => (
                                    <Badge key={tag} variant="outline" className="text-xs">
                                      {tag}
                                    </Badge>
                                  ))}
                                </div>
                              )}
                            </div>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="text-muted-foreground hover:text-destructive"
                              onClick={(e) => {
                                e.stopPropagation();
                                deleteRecording(recording.id);
                              }}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Stations View */}
            {view === "stations" && (
              <div className="space-y-6">
                <h1 className="text-2xl font-bold">放送局</h1>

                <Tabs defaultValue="nhk" onValueChange={(v) => {
                  setServiceType(v as ServiceType);
                  setSelectedStation(null);
                  setPrograms([]);
                }}>
                  <TabsList className="mb-6">
                    <TabsTrigger value="nhk" className="px-6">NHK</TabsTrigger>
                    <TabsTrigger value="radiko" className="px-6">radiko</TabsTrigger>
                  </TabsList>

                  <TabsContent value="nhk">
                    <div className="grid md:grid-cols-[280px_1fr] gap-6">
                      <Card>
                        <CardHeader>
                          <CardTitle className="text-base">局を選択</CardTitle>
                        </CardHeader>
                        <CardContent className="p-2">
                          <div className="space-y-1">
                            {nhkStations.map((station) => (
                              <Button
                                key={station.id}
                                variant={selectedStation?.id === station.id ? "default" : "ghost"}
                                className="w-full justify-start"
                                onClick={() => handleStationSelect(station)}
                              >
                                <Radio className="h-4 w-4 mr-2" />
                                {station.name}
                              </Button>
                            ))}
                          </div>
                        </CardContent>
                      </Card>

                      <Card>
                        <CardHeader>
                          <CardTitle className="text-base">
                            {selectedStation ? `${selectedStation.name} - 番組表` : "番組表"}
                          </CardTitle>
                        </CardHeader>
                        <CardContent>
                          {!selectedStation ? (
                            <div className="text-center py-12">
                              <Radio className="h-12 w-12 text-muted-foreground/50 mx-auto mb-4" />
                              <p className="text-muted-foreground">放送局を選択してください</p>
                            </div>
                          ) : isProgramLoading ? (
                            <div className="flex items-center justify-center py-12">
                              <Loader2 className="h-8 w-8 animate-spin text-primary" />
                            </div>
                          ) : programs.length === 0 ? (
                            <div className="text-center py-12">
                              <AlertCircle className="h-12 w-12 text-muted-foreground/50 mx-auto mb-4" />
                              <p className="text-muted-foreground">番組情報を取得できませんでした</p>
                              <p className="text-sm text-muted-foreground mt-1">
                                しばらく待ってから再度お試しください
                              </p>
                            </div>
                          ) : (
                            <ScrollArea className="max-h-[500px]">
                              <div className="space-y-2">
                                {programs.map((program) => (
                                  <div
                                    key={program.id}
                                    className="flex items-center gap-4 p-3 rounded-lg hover:bg-secondary/50 transition-colors"
                                  >
                                    <div className="text-sm text-muted-foreground w-24 flex-shrink-0">
                                      {formatTime(program.start_time)} - {formatTime(program.end_time)}
                                    </div>
                                    <div className="flex-1 min-w-0">
                                      <p className="font-medium truncate">{program.title}</p>
                                      {program.performer && (
                                        <p className="text-sm text-muted-foreground truncate">
                                          {program.performer}
                                        </p>
                                      )}
                                    </div>
                                    <Button
                                      size="sm"
                                      onClick={() => {
                                        const start = new Date(
                                          `${program.start_time.slice(0, 4)}-${program.start_time.slice(4, 6)}-${program.start_time.slice(6, 8)}T${program.start_time.slice(8, 10)}:${program.start_time.slice(10, 12)}:00`
                                        );
                                        const end = new Date(
                                          `${program.end_time.slice(0, 4)}-${program.end_time.slice(4, 6)}-${program.end_time.slice(6, 8)}T${program.end_time.slice(8, 10)}:${program.end_time.slice(10, 12)}:00`
                                        );
                                        const durationMinutes = Math.ceil((end.getTime() - start.getTime()) / 60000);
                                        startRecording(selectedStation!, durationMinutes, program.title);
                                      }}
                                    >
                                      <Mic className="h-3 w-3 mr-1.5" />
                                      録音
                                    </Button>
                                  </div>
                                ))}
                              </div>
                            </ScrollArea>
                          )}
                        </CardContent>
                      </Card>
                    </div>
                  </TabsContent>

                  <TabsContent value="radiko">
                    {!isAuthenticated ? (
                      <Card className="max-w-md mx-auto">
                        <CardHeader className="text-center">
                          <CardTitle>radikoにログイン</CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-4">
                          <div className="space-y-2">
                            <Input
                              type="email"
                              placeholder="メールアドレス"
                              value={email}
                              onChange={(e) => setEmail(e.target.value)}
                            />
                          </div>
                          <div className="space-y-2">
                            <Input
                              type="password"
                              placeholder="パスワード"
                              value={password}
                              onChange={(e) => setPassword(e.target.value)}
                            />
                          </div>
                          <Button
                            className="w-full"
                            onClick={handleLogin}
                            disabled={isLoading}
                          >
                            {isLoading ? (
                              <>
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                                ログイン中...
                              </>
                            ) : (
                              "ログイン"
                            )}
                          </Button>
                          <p className="text-xs text-center text-muted-foreground">
                            ※ 無料エリアのみの場合は空欄でもOK
                          </p>
                        </CardContent>
                      </Card>
                    ) : (
                      <div className="grid md:grid-cols-[280px_1fr] gap-6">
                        <Card>
                          <CardHeader>
                            <CardTitle className="text-base">局を選択</CardTitle>
                          </CardHeader>
                          <CardContent className="p-2">
                            <ScrollArea className="max-h-[400px]">
                              <div className="space-y-1">
                                {stations.map((station) => (
                                  <Button
                                    key={station.id}
                                    variant={selectedStation?.id === station.id ? "default" : "ghost"}
                                    className="w-full justify-start"
                                    onClick={() => handleStationSelect(station)}
                                  >
                                    <Radio className="h-4 w-4 mr-2" />
                                    {station.name}
                                  </Button>
                                ))}
                              </div>
                            </ScrollArea>
                          </CardContent>
                        </Card>

                        <Card>
                          <CardHeader>
                            <CardTitle className="text-base">
                              {selectedStation ? `${selectedStation.name} - 番組表` : "番組表"}
                            </CardTitle>
                          </CardHeader>
                          <CardContent>
                            {!selectedStation ? (
                              <div className="text-center py-12">
                                <Radio className="h-12 w-12 text-muted-foreground/50 mx-auto mb-4" />
                                <p className="text-muted-foreground">放送局を選択してください</p>
                              </div>
                            ) : isProgramLoading ? (
                              <div className="flex items-center justify-center py-12">
                                <Loader2 className="h-8 w-8 animate-spin text-primary" />
                              </div>
                            ) : programs.length === 0 ? (
                              <div className="text-center py-12">
                                <AlertCircle className="h-12 w-12 text-muted-foreground/50 mx-auto mb-4" />
                                <p className="text-muted-foreground">番組情報を取得できませんでした</p>
                              </div>
                            ) : (
                              <ScrollArea className="max-h-[500px]">
                                <div className="space-y-2">
                                  {programs.map((program) => (
                                    <div
                                      key={program.id}
                                      className="flex items-center gap-4 p-3 rounded-lg hover:bg-secondary/50 transition-colors"
                                    >
                                      <div className="text-sm text-muted-foreground w-24 flex-shrink-0">
                                        {formatTime(program.start_time)} - {formatTime(program.end_time)}
                                      </div>
                                      <div className="flex-1 min-w-0">
                                        <p className="font-medium truncate">{program.title}</p>
                                        {program.performer && (
                                          <p className="text-sm text-muted-foreground truncate">
                                            {program.performer}
                                          </p>
                                        )}
                                      </div>
                                      <Button
                                        size="sm"
                                        onClick={() => {
                                          const start = new Date(
                                            `${program.start_time.slice(0, 4)}-${program.start_time.slice(4, 6)}-${program.start_time.slice(6, 8)}T${program.start_time.slice(8, 10)}:${program.start_time.slice(10, 12)}:00`
                                          );
                                          const end = new Date(
                                            `${program.end_time.slice(0, 4)}-${program.end_time.slice(4, 6)}-${program.end_time.slice(6, 8)}T${program.end_time.slice(8, 10)}:${program.end_time.slice(10, 12)}:00`
                                          );
                                          const durationMinutes = Math.ceil((end.getTime() - start.getTime()) / 60000);
                                          startRecording(selectedStation!, durationMinutes, program.title);
                                        }}
                                      >
                                        <Mic className="h-3 w-3 mr-1.5" />
                                        録音
                                      </Button>
                                    </div>
                                  ))}
                                </div>
                              </ScrollArea>
                            )}
                          </CardContent>
                        </Card>
                      </div>
                    )}
                  </TabsContent>
                </Tabs>
              </div>
            )}

            {/* Schedule View */}
            {view === "schedule" && (
              <div className="space-y-6">
                <h1 className="text-2xl font-bold">予約録音</h1>
                <Card>
                  <CardContent className="flex flex-col items-center justify-center py-16">
                    <Calendar className="h-12 w-12 text-muted-foreground/50 mb-4" />
                    <p className="text-muted-foreground">この機能は準備中です</p>
                    <p className="text-sm text-muted-foreground mt-1">
                      今後のアップデートにご期待ください
                    </p>
                  </CardContent>
                </Card>
              </div>
            )}

            {/* Settings View */}
            {view === "settings" && (
              <div className="space-y-6">
                <h1 className="text-2xl font-bold">設定</h1>
                <Card>
                  <CardContent className="p-6 space-y-6">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-medium">再生速度</p>
                        <p className="text-sm text-muted-foreground">
                          録音の再生速度を調整します
                        </p>
                      </div>
                      <Select
                        value={String(playbackSpeed)}
                        onValueChange={(v) => setPlaybackSpeed(Number(v))}
                        className="w-32"
                      >
                        <option value="0.5">0.5x</option>
                        <option value="0.75">0.75x</option>
                        <option value="1">1x</option>
                        <option value="1.25">1.25x</option>
                        <option value="1.5">1.5x</option>
                        <option value="2">2x</option>
                      </Select>
                    </div>
                  </CardContent>
                </Card>
              </div>
            )}
          </div>
        </ScrollArea>
      </main>
    </div>
  );
}

export default App;
