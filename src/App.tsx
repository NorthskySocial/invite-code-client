import {useState, useEffect} from 'react';
import {apiService, InviteCode} from './api';
import {
  LogIn,
  Plus,
  Download,
  Trash2,
  RefreshCw,
  LogOut,
  ShieldCheck,
  QrCode,
  Copy,
  Check,
  Filter,
  Sun,
  Moon
} from 'lucide-react';
import {format, isValid} from 'date-fns';

const formatDate = (dateString: string | undefined) => {
  if (!dateString) {
    return '-';
  }
  const date = new Date(dateString);
  if (!isValid(date)) {
    return 'Invalid Date';
  }
  return format(date, 'MMM d, yyyy HH:mm');
};

type Page = 'Home' | 'Login' | 'QrVerify' | 'QrValidate';
type FilterStatus = 'All' | 'Used' | 'Unused' | 'Disabled';

function App() {
  const [page, setPage] = useState<Page>('Login');
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
  const [darkMode, setDarkMode] = useState<boolean>(() => {
    const saved = localStorage.getItem('theme');
    return saved === 'dark' || (!saved && window.matchMedia('(prefers-color-scheme: dark)').matches);
  });
  const [invites, setInvites] = useState<InviteCode[]>([]);
  const [filteredInvites, setFilteredInvites] = useState<InviteCode[]>([]);
  const [filter, setFilter] = useState<FilterStatus>('All');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [_twoFactorToken, setTwoFactorToken] = useState<string | null>(null);
  const [otpToken, setOtpToken] = useState('');
  const [qrCode, setQrCode] = useState<string | null>(null);
  const [inviteCount, setInviteCount] = useState(1);
  const [copied, setCopied] = useState<string | null>(null);

  useEffect(() => {
    if (token) {
      setPage('Home');
      fetchInvites();
    }
  }, [token]);

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }
  }, [darkMode]);

  useEffect(() => {
    applyFilter();
  }, [invites, filter]);

  const fetchInvites = async () => {
    setLoading(true);
    try {
      const response = await apiService.getInviteCodes();
      const data = response.data?.codes || [];
      setInvites(Array.isArray(data) ? data : []);
      setError(null);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to fetch invites');
      if (err.response?.status === 401) {
        handleLogout();
      }
    } finally {
      setLoading(false);
    }
  };

  const getStatus = (invite: InviteCode): FilterStatus => {
    if (invite.disabled) {
      return 'Disabled';
    }
    if (invite.available === 0) {
      return 'Used';
    }
    return 'Unused';
  };

  const getUsedAt = (invite: InviteCode): string | undefined => {
    if (invite.uses && invite.uses.length > 0) {
      return invite.uses[0].usedAt;
    }
    return undefined;
  };

  const applyFilter = () => {
    if (filter === 'All') {
      setFilteredInvites(invites);
    } else {
      setFilteredInvites(invites.filter(i => getStatus(i) === filter));
    }
  };

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setTwoFactorToken(null);
    setOtpToken('');
    setQrCode(null);
    try {
      const response = await apiService.login(username, password);
      if (response.data.requires_2fa || (response.data.otp_enabled && response.data.otp_verified && !response.data.token)) {
        setTwoFactorToken(response.data.two_factor_token || null);
        setPage('QrValidate');
        return;
      }
      if (response.data.token) {
        localStorage.setItem('token', response.data.token);
        setToken(response.data.token);

        // If OTP is enabled but not verified, redirect to setup
        if (response.data.otp_enabled && !response.data.otp_verified) {
          if (response.data.otp_auth_url) {
            const qrCodeUrl = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(response.data.otp_auth_url)}`;
            setQrCode(qrCodeUrl);
            setPage('QrVerify');
          } else {
            setPage('Home');
          }
        } else {
          setPage('Home');
        }
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  const handleLogout = () => {
    localStorage.removeItem('token');
    setToken(null);
    setTwoFactorToken(null);
    setQrCode(null);
    setOtpToken('');
    setError(null);
    setPage('Login');
  };

  const handleCreateInvites = async () => {
    setLoading(true);
    setError(null);
    try {
      await apiService.createInviteCodes(inviteCount);
      fetchInvites();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to create invites');
    } finally {
      setLoading(false);
    }
  };

  const handleDisableInvite = async (code: string) => {
    setError(null);
    try {
      await apiService.disableInviteCode(code);
      fetchInvites();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to disable invite');
    }
  };

  const handleGenerateOtp = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await apiService.generateOtp();
      setQrCode(response.data.qr_code);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to generate QR');
    } finally {
      setLoading(false);
    }
  };

  const handleVerifyOtp = async () => {
    setLoading(true);
    setError(null);
    try {
      await apiService.verifyOtp(otpToken);
      alert('OTP Verified successfully');
      setPage('Home');
    } catch (err: any) {
      setError(err.response?.data?.error || 'OTP Verification failed');
    } finally {
      setLoading(false);
    }
  };

  const handleValidateOtp = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await apiService.validateOtp(otpToken);
      const token = (response.data as any).token;
      if (token) {
        localStorage.setItem('token', token);
        setToken(token);
        alert('OTP Validated successfully');
        setPage('Home');
      } else {
        alert('OTP Validated successfully');
        setPage('Home');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'OTP Validation failed');
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(text);
    setTimeout(() => setCopied(null), 2000);
  };

  const downloadTxt = () => {
    const content = filteredInvites.map(i => i.code).join('\n');
    const blob = new Blob([content], {type: 'text/plain'});
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `invites_${format(new Date(), 'yyyyMMdd_HHmmss')}.txt`;
    a.click();
  };

  if (page === 'Login') {
    return (
      <div
        className="min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center p-4 transition-colors duration-200">
        <div
          className="bg-white dark:bg-gray-800 p-8 rounded-lg shadow-md w-full max-w-md border dark:border-gray-700">
          <div className="flex flex-col items-center mb-6 relative">
            <button
              onClick={() => setDarkMode(!darkMode)}
              className="absolute right-0 top-0 p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-full transition"
              aria-label="Toggle Theme"
            >
              {darkMode ? <Sun className="w-5 h-5"/> : <Moon className="w-5 h-5"/>}
            </button>
            <div className="bg-blue-600 p-3 rounded-full mb-4">
              <LogIn className="text-white w-8 h-8"/>
            </div>
            <h1 className="text-2xl font-bold text-gray-800 dark:text-white">Invites Client</h1>
            <p className="text-gray-500 dark:text-gray-400 text-center mt-2">Enter your credentials
              to access the manager</p>
          </div>

          <form onSubmit={handleLogin}>
            <div className="mb-4">
              <input
                type="text"
                placeholder="Username"
                className="w-full p-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded focus:ring-2 focus:ring-blue-500 outline-none"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                autoFocus
              />
            </div>
            <div className="mb-4">
              <input
                type="password"
                placeholder="Password"
                className="w-full p-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded focus:ring-2 focus:ring-blue-500 outline-none"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </div>
            {error && <p className="text-red-500 text-sm mb-4">{error}</p>}
            <button
              type="submit"
              disabled={loading}
              className="w-full bg-blue-600 text-white p-3 rounded font-semibold hover:bg-blue-700 transition disabled:opacity-50"
            >
              {loading ? 'Logging in...' : 'Login'}
            </button>
          </form>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 w-full transition-colors duration-200">
      {/* Header */}
      <nav
        className="bg-white dark:bg-gray-800 shadow-sm border-b dark:border-gray-700 px-4 py-3 flex justify-between items-center sticky top-0 z-10">
        <div className="flex items-center gap-2">
          <ShieldCheck className="text-blue-600 w-6 h-6"/>
          <span className="font-bold text-lg text-gray-800 dark:text-white">Invites Manager</span>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setDarkMode(!darkMode)}
            className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
            title="Toggle Theme"
          >
            {darkMode ? <Sun className="w-5 h-5"/> : <Moon className="w-5 h-5"/>}
          </button>
          <button
            onClick={() => {
              setError(null);
              handleGenerateOtp();
              setPage('QrVerify');
            }}
            className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
            title="Setup OTP"
          >
            <QrCode className="w-5 h-5"/>
          </button>
          <button
            onClick={() => {
              setError(null);
              setPage('QrValidate');
            }}
            className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
            title="Validate OTP"
          >
            <ShieldCheck className="w-5 h-5"/>
          </button>
          <button
            onClick={handleLogout}
            className="p-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
            title="Logout"
          >
            <LogOut className="w-5 h-5"/>
          </button>
        </div>
      </nav>

      <main className="max-w-6xl mx-auto p-4 md:p-6">
        {page === 'Home' && (
          <div className="space-y-6">
            {/* Controls */}
            <div
              className="bg-white dark:bg-gray-800 p-4 rounded-lg shadow-sm border dark:border-gray-700 flex flex-wrap gap-4 items-center justify-between">
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min="1"
                  max="100"
                  className="w-20 p-2 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded"
                  value={inviteCount}
                  onChange={(e) => setInviteCount(parseInt(e.target.value))}
                />
                <button
                  onClick={handleCreateInvites}
                  disabled={loading}
                  className="bg-blue-600 text-white px-4 py-2 rounded flex items-center gap-2 hover:bg-blue-700 transition"
                >
                  <Plus className="w-4 h-4"/> Generate
                </button>
              </div>

              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <Filter className="w-4 h-4 text-gray-400"/>
                  <select
                    value={filter}
                    onChange={(e) => setFilter(e.target.value as FilterStatus)}
                    className="p-2 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded"
                  >
                    <option value="All">All Status</option>
                    <option value="Unused">Unused</option>
                    <option value="Used">Used</option>
                    <option value="Disabled">Disabled</option>
                  </select>
                </div>

                <button
                  onClick={downloadTxt}
                  className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded border dark:border-gray-600 flex items-center gap-2"
                >
                  <Download className="w-4 h-4"/> Export
                </button>

                <button
                  onClick={fetchInvites}
                  disabled={loading}
                  className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded border dark:border-gray-600"
                >
                  <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`}/>
                </button>
              </div>
            </div>

            {error &&
                <p className="text-red-500 bg-red-50 dark:bg-red-900/20 p-3 rounded border border-red-200 dark:border-red-900/30">{error}</p>}

            {/* Table */}
            <div
              className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border dark:border-gray-700 overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full text-left">
                  <thead className="bg-gray-50 dark:bg-gray-700/50 border-b dark:border-gray-700">
                  <tr>
                    <th
                      className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">Invite
                      Code
                    </th>
                    <th
                      className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">Status
                    </th>
                    <th
                      className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">Created
                      At
                    </th>
                    <th
                      className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">Used
                      At
                    </th>
                    <th
                      className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider text-right">Actions
                    </th>
                  </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-100 dark:divide-gray-700">
                  {filteredInvites.map((invite) => {
                    const status = getStatus(invite);
                    const usedAt = getUsedAt(invite);
                    return (
                      <tr key={invite.code}
                        className="hover:bg-gray-50 dark:hover:bg-gray-700/30 transition">
                      <td
                        className="px-6 py-4 font-mono text-sm flex items-center gap-2 dark:text-gray-200">
                        {invite.code}
                        <button
                          onClick={() => copyToClipboard(invite.code)}
                          className="text-gray-400 hover:text-blue-600"
                        >
                          {copied === invite.code ? <Check className="w-4 h-4 text-green-500"/> :
                            <Copy className="w-4 h-4"/>}
                        </button>
                      </td>
                      <td className="px-6 py-4">
                          <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                            status === 'Unused' ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' :
                              status === 'Used' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400' :
                                'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
                          }`}>
                            {status}
                          </span>
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                        {formatDate(invite.createdAt)}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                        {formatDate(usedAt)}
                      </td>
                      <td className="px-6 py-4 text-right">
                        {status === 'Unused' && (
                          <button
                            onClick={() => handleDisableInvite(invite.code)}
                            className="text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 p-1 rounded transition"
                            title="Disable"
                          >
                            <Trash2 className="w-4 h-4"/>
                          </button>
                        )}
                      </td>
                    </tr>
                    );
                  })}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        )}

        {page === 'QrVerify' && (
          <div className="max-w-md mx-auto space-y-6 text-center">
            <h2 className="text-xl font-bold dark:text-white">Setup Multi-Factor Authentication</h2>
            {qrCode && (
              <div className="bg-white p-4 rounded-lg shadow-md inline-block">
                <img src={qrCode} alt="OTP QR Code" className="mx-auto"/>
              </div>
            )}
            <p className="text-gray-600 dark:text-gray-400">Scan this QR code with your
              authenticator app, then enter the code below to verify.</p>
            <div className="space-y-4">
              <input
                type="text"
                placeholder="000000"
                className="w-full p-3 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded text-center text-2xl tracking-widest"
                value={otpToken}
                onChange={(e) => setOtpToken(e.target.value)}
              />
              {error && <p className="text-red-500 text-sm mb-4">{error}</p>}
              <div className="flex gap-3">
                <button
                  onClick={() => {
                    setError(null);
                    setPage('Home');
                  }}
                  className="flex-1 p-3 border dark:border-gray-600 dark:text-white rounded font-semibold hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  Cancel
                </button>
                <button
                  onClick={handleVerifyOtp}
                  disabled={loading}
                  className="flex-1 bg-blue-600 text-white p-3 rounded font-semibold hover:bg-blue-700"
                >
                  {loading ? 'Verifying...' : 'Verify'}
                </button>
              </div>
            </div>
          </div>
        )}

        {page === 'QrValidate' && (
          <div className="max-w-md mx-auto space-y-6 text-center">
            <h2 className="text-xl font-bold dark:text-white">Two-Factor Authentication</h2>
            <p className="text-gray-600 dark:text-gray-400">Please enter the 6-digit code from your
              authenticator app.</p>
            <div className="space-y-4">
              <input
                type="text"
                placeholder="000000"
                className="w-full p-3 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded text-center text-2xl tracking-widest"
                value={otpToken}
                onChange={(e) => setOtpToken(e.target.value)}
              />
              {error && <p className="text-red-500 text-sm mb-4">{error}</p>}
              <div className="flex gap-3">
                <button
                  onClick={() => {
                    setError(null);
                    if (token) {
                      setPage('Home');
                    } else {
                      setPage('Login');
                    }
                  }}
                  className="flex-1 p-3 border dark:border-gray-600 dark:text-white rounded font-semibold hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  Cancel
                </button>
                <button
                  onClick={handleValidateOtp}
                  disabled={loading}
                  className="flex-1 bg-blue-600 text-white p-3 rounded font-semibold hover:bg-blue-700"
                >
                  {loading ? 'Validating...' : 'Validate'}
                </button>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
