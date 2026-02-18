import { useState, useEffect, useCallback } from 'react';
import {
  apiService,
  mockApiService,
  updateApiBaseURL,
  InviteCode,
  Admin,
  isAxiosError,
} from './api';
import {
  LogIn,
  Plus,
  Download,
  Trash2,
  RefreshCw,
  LogOut,
  ShieldCheck,
  Copy,
  Check,
  Filter,
  Sun,
  Moon,
  Globe,
  Zap,
  Users,
  UserPlus,
  UserMinus,
} from 'lucide-react';
import { format, isValid } from 'date-fns';

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

type Page = 'Home' | 'Login' | 'QrVerify' | 'QrValidate' | 'Admins';
type FilterStatus = 'All' | 'Used' | 'Unused' | 'Disabled';

function App() {
  const [page, setPage] = useState<Page>(() => {
    const token = localStorage.getItem('token');
    return token ? 'Home' : 'Login';
  });
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
  const [darkMode, setDarkMode] = useState<boolean>(() => {
    const saved = localStorage.getItem('theme');
    return (
      saved === 'dark' || (!saved && window.matchMedia('(prefers-color-scheme: dark)').matches)
    );
  });
  const [invites, setInvites] = useState<InviteCode[]>([]);
  const [filteredInvites, setFilteredInvites] = useState<InviteCode[]>([]);
  const [filter, setFilter] = useState<FilterStatus>('All');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [apiHost, setApiHost] = useState(
    localStorage.getItem('api_host') ||
      import.meta.env.VITE_API_HOST ||
      'https://frontend.myapp.local/'
  );
  const [isDemoMode, setIsDemoMode] = useState(localStorage.getItem('demo_mode') === 'true');
  const activeService = isDemoMode ? mockApiService : apiService;
  const [_twoFactorToken, setTwoFactorToken] = useState<string | null>(null);
  const [otpToken, setOtpToken] = useState('');
  const [qrCode, setQrCode] = useState<string | null>(null);
  const [inviteCount, setInviteCount] = useState(1);
  const [copied, setCopied] = useState<string | null>(null);
  const [handles, setHandles] = useState<Record<string, string>>({});
  const [admins, setAdmins] = useState<Admin[]>([]);
  const [newAdminUsername, setNewAdminUsername] = useState('');
  const [newAdminPassword, setNewAdminPassword] = useState<string | null>(null);

  const getStatus = (invite: InviteCode): FilterStatus => {
    if (invite.disabled) {
      return 'Disabled';
    }
    if (invite.available === 0 || (invite.uses && invite.uses.length > 0)) {
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

  const getUsedBy = (invite: InviteCode): string | undefined => {
    if (invite.uses && invite.uses.length > 0) {
      return invite.uses[0].usedBy;
    }
    return undefined;
  };

  const resolveHandles = useCallback(
    async (invitesList: InviteCode[]) => {
      const didsToResolve = new Set<string>();
      invitesList.forEach((invite) => {
        const usedBy = getUsedBy(invite);
        if (usedBy && usedBy.startsWith('did:') && !handles[usedBy]) {
          didsToResolve.add(usedBy);
        }
      });

      for (const did of didsToResolve) {
        try {
          const response = await activeService.resolveDid(did);
          // PLC directory response has 'alsoKnownAs' array with 'at://handle'
          const alsoKnownAs = response.data?.alsoKnownAs || [];
          const handleUri = alsoKnownAs.find((uri: string) => uri.startsWith('at://'));
          if (handleUri) {
            const handle = handleUri.replace('at://', '');
            setHandles((prev) => ({ ...prev, [did]: handle }));
          } else {
            // If no handle found, we might want to store the DID itself or a placeholder
            setHandles((prev) => ({ ...prev, [did]: did }));
          }
        } catch (err) {
          console.error(`Failed to resolve DID: ${did}`, err);
          // Optionally store the DID so we don't keep trying
          setHandles((prev) => ({ ...prev, [did]: did }));
        }
      }
    },
    [activeService, handles]
  );

  const fetchInvites = useCallback(async () => {
    setLoading(true);
    try {
      const response = await activeService.getInviteCodes();
      const data = response.data?.codes || [];
      setInvites(Array.isArray(data) ? data : []);
      setError(null);
      if (Array.isArray(data)) {
        resolveHandles(data);
      }
    } catch (err: unknown) {
      setError(
        isAxiosError(err)
          ? err.response?.data?.error || 'Failed to fetch invites'
          : 'An error occurred'
      );
      if (isAxiosError(err) && err.response?.status === 401) {
        handleLogout();
      }
    } finally {
      setLoading(false);
    }
  }, [activeService, resolveHandles]);

  const fetchAdmins = useCallback(async () => {
    setLoading(true);
    try {
      const response = await activeService.getAdmins();
      setAdmins(response.data?.admins || []);
      setError(null);
    } catch (err: unknown) {
      console.error('Failed to fetch admins', err);
      setError(
        isAxiosError(err)
          ? err.response?.data?.error || 'Failed to fetch admins'
          : 'An error occurred'
      );
    } finally {
      setLoading(false);
    }
  }, [activeService, setAdmins, setError]);

  const handleAddAdmin = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedUsername = newAdminUsername.trim();
    if (!trimmedUsername) {
      return;
    }
    setLoading(true);
    setError(null);
    setNewAdminPassword(null);
    try {
      const response = await activeService.addAdmin(trimmedUsername);
      setNewAdminUsername('');
      if (response.data.password) {
        setNewAdminPassword(response.data.password);
      }
      await fetchAdmins();
    } catch (err: unknown) {
      setError(
        isAxiosError(err) ? err.response?.data?.error || 'Failed to add admin' : 'An error occurred'
      );
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveAdmin = async (username: string) => {
    if (!confirm(`Are you sure you want to remove ${username} as an admin?`)) {
      return;
    }
    setLoading(true);
    setError(null);
    try {
      await activeService.removeAdmin(username);
      await fetchAdmins();
    } catch (err: unknown) {
      setError(
        isAxiosError(err)
          ? err.response?.data?.error || 'Failed to remove admin'
          : 'An error occurred'
      );
    } finally {
      setLoading(false);
    }
  };

  const applyFilter = useCallback(() => {
    if (filter === 'All') {
      setFilteredInvites(invites);
    } else {
      setFilteredInvites(invites.filter((i) => getStatus(i) === filter));
    }
  }, [invites, filter]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setTwoFactorToken(null);
    setOtpToken('');
    setQrCode(null);

    // Normalize and update API host
    let normalizedHost = apiHost.trim();
    if (normalizedHost && !normalizedHost.endsWith('/')) {
      normalizedHost += '/';
    }
    if (normalizedHost) {
      localStorage.setItem('api_host', normalizedHost);
      updateApiBaseURL(normalizedHost);
    }

    localStorage.setItem('demo_mode', isDemoMode.toString());

    try {
      const response = await activeService.login(username, password);
      if (response.data.otp_enabled && response.data.otp_verified && !response.data.token) {
        setTwoFactorToken(response.data.two_factor_token || null);
        setPage('QrValidate');
        return;
      }

      // If OTP is enabled but not verified, OR if OTP is disabled and not verified (new setup)
      if (
        (response.data.otp_enabled && !response.data.otp_verified) ||
        (!response.data.otp_enabled && !response.data.otp_verified)
      ) {
        if (response.data.otp_auth_url) {
          const qrCodeUrl = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(response.data.otp_auth_url)}`;
          setQrCode(qrCodeUrl);
          setPage('QrVerify');
        } else {
          if (response.data.token) {
            localStorage.setItem('token', response.data.token);
            setToken(response.data.token);
          }
          setPage('Home');
        }
      } else {
        if (response.data.token) {
          localStorage.setItem('token', response.data.token);
          setToken(response.data.token);
        }
        setPage('Home');
      }
    } catch (err: unknown) {
      setError(
        isAxiosError(err) ? err.response?.data?.error || 'Login failed' : 'An error occurred'
      );
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
      await activeService.createInviteCodes(inviteCount);
      await fetchInvites();
    } catch (err: unknown) {
      setError(
        isAxiosError(err)
          ? err.response?.data?.error || 'Failed to create invites'
          : 'An error occurred'
      );
    } finally {
      setLoading(false);
    }
  };

  const handleDisableInvite = async (code: string) => {
    setError(null);
    try {
      await activeService.disableInviteCode(code);
      await fetchInvites();
    } catch (err: unknown) {
      setError(
        isAxiosError(err)
          ? err.response?.data?.error || 'Failed to disable invite'
          : 'An error occurred'
      );
    }
  };

  const handleVerifyOtp = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await activeService.verifyOtp(otpToken);
      const token = (response.data as unknown as { token?: string }).token;
      if (token) {
        localStorage.setItem('token', token);
        setToken(token);
      }
      alert('OTP Verified successfully');
      setPage('Home');
    } catch (err: unknown) {
      setError(
        isAxiosError(err)
          ? err.response?.data?.error || 'OTP Verification failed'
          : 'An error occurred'
      );
    } finally {
      setLoading(false);
    }
  };

  const handleValidateOtp = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await activeService.validateOtp(otpToken);
      const token = (response.data as unknown as { token?: string }).token;
      if (token) {
        localStorage.setItem('token', token);
        setToken(token);
        alert('OTP Validated successfully');
        setPage('Home');
      } else {
        alert('OTP Validated successfully');
        setPage('Home');
      }
    } catch (err: unknown) {
      setError(
        isAxiosError(err)
          ? err.response?.data?.error || 'OTP Validation failed'
          : 'An error occurred'
      );
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(text);
    setTimeout(() => setCopied(null), 2000);
  };

  const downloadCsv = () => {
    const headers = ['Invite Code', 'Status', 'Created At', 'Used By', 'Used At'];
    const rows = filteredInvites.map((invite) => {
      const usedBy = getUsedBy(invite);
      const resolvedHandle = usedBy ? handles[usedBy] : null;
      const usedByText = resolvedHandle || usedBy || '-';

      return [
        invite.code,
        getStatus(invite),
        formatDate(invite.createdAt),
        usedByText,
        formatDate(getUsedAt(invite)),
      ]
        .map((val) => `"${String(val).replace(/"/g, '""')}"`)
        .join(',');
    });

    const csvContent = [headers.join(','), ...rows].join('\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `invites_${format(new Date(), 'yyyyMMdd_HHmmss')}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const isAdminPage = page === 'Admins';
  useEffect(() => {
    if (token) {
      fetchInvites();
      fetchAdmins();
    }
  }, [token, isAdminPage, fetchInvites, fetchAdmins]);

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
  }, [invites, filter, applyFilter]);

  if (page === 'Login') {
    return (
      <div className="min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center p-4 transition-colors duration-200">
        <div className="bg-white dark:bg-gray-800 p-6 md:p-8 rounded-xl shadow-lg w-full max-w-md border dark:border-gray-700">
          <div className="flex flex-col items-center mb-8 relative">
            <button
              onClick={() => setDarkMode(!darkMode)}
              className="absolute right-0 top-0 p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-full transition"
              aria-label="Toggle Theme"
            >
              {darkMode ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
            </button>
            <div className="bg-blue-600 p-4 rounded-2xl mb-4 shadow-blue-500/20 shadow-lg">
              <LogIn className="text-white w-8 h-8" />
            </div>
            <h1 className="text-2xl font-bold text-gray-800 dark:text-white">Invites Client</h1>
            <p className="text-gray-500 dark:text-gray-400 text-center mt-2">
              Enter your credentials to access the manager
            </p>
          </div>

          <form onSubmit={handleLogin} className="space-y-4">
            <div className="flex items-center gap-3 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-900/30 rounded-lg mb-2">
              <div className="flex-1">
                <p className="text-sm font-medium text-blue-800 dark:text-blue-300">Demo Mode</p>
                <p className="text-xs text-blue-600 dark:text-blue-400">
                  Run locally without a backend
                </p>
              </div>
              <button
                type="button"
                onClick={() => setIsDemoMode(!isDemoMode)}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ring-2 ring-offset-2 ring-transparent focus:ring-blue-500 ${isDemoMode ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'}`}
                role="switch"
                aria-checked={isDemoMode}
                aria-label="Demo Mode"
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${isDemoMode ? 'translate-x-6' : 'translate-x-1'}`}
                />
              </button>
            </div>
            {!isDemoMode && (
              <div className="relative">
                <Globe className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 w-5 h-5" />
                <input
                  type="text"
                  placeholder="Backend API Host (e.g. https://api.example.com)"
                  className="w-full p-3.5 pl-10 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                  value={apiHost}
                  onChange={(e) => setApiHost(e.target.value)}
                />
              </div>
            )}
            <div>
              <input
                type="text"
                placeholder="Username"
                className="w-full p-3.5 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                autoFocus
              />
            </div>
            <div>
              <input
                type="password"
                placeholder="Password"
                className="w-full p-3.5 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </div>
            {error && (
              <div className="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-900/30 p-3 rounded-lg">
                <p className="text-red-500 text-sm text-center">{error}</p>
              </div>
            )}
            <button
              type="submit"
              disabled={loading}
              className="w-full bg-blue-600 text-white p-3.5 rounded-lg font-bold hover:bg-blue-700 transition shadow-lg shadow-blue-500/20 disabled:opacity-50 active:scale-[0.98] flex items-center justify-center gap-2"
            >
              {isDemoMode && <Zap className="w-5 h-5" />}
              {loading ? 'Logging in...' : isDemoMode ? 'Start Demo' : 'Login'}
            </button>
          </form>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 w-full transition-colors duration-200">
      {/* Header */}
      <nav className="bg-white dark:bg-gray-800 shadow-sm border-b dark:border-gray-700 px-4 py-3 sticky top-0 z-10">
        <div className="max-w-6xl mx-auto flex justify-between items-center">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <ShieldCheck className="text-blue-600 w-6 h-6 md:w-7 md:h-7" />
              <span className="font-bold text-base md:text-lg text-gray-800 dark:text-white whitespace-nowrap hidden sm:inline">
                Invites Manager
              </span>
            </div>
            <div className="flex items-center space-x-1">
              <button
                onClick={() => setPage('Home')}
                className={`px-3 py-1.5 rounded-md text-sm font-medium transition ${page === 'Home' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300' : 'text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}`}
              >
                Invites
              </button>
              <button
                onClick={() => setPage('Admins')}
                className={`px-3 py-1.5 rounded-md text-sm font-medium transition ${page === 'Admins' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300' : 'text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}`}
              >
                Admins
              </button>
            </div>
          </div>
          <div className="flex gap-1 md:gap-2">
            <button
              onClick={() => setDarkMode(!darkMode)}
              className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition min-w-[40px] min-h-[40px] flex items-center justify-center"
              title="Toggle Theme"
            >
              {darkMode ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
            </button>
            <button
              onClick={handleLogout}
              className="p-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded min-w-[40px] min-h-[40px] flex items-center justify-center"
              title="Logout"
            >
              <LogOut className="w-5 h-5" />
            </button>
          </div>
        </div>
      </nav>

      <main className="max-w-6xl mx-auto p-4 md:p-6">
        {page === 'Admins' && (
          <div className="space-y-6">
            <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm border dark:border-gray-700 p-6">
              <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
                <div>
                  <h2 className="text-xl font-bold text-gray-800 dark:text-white flex items-center gap-2">
                    <Users className="w-5 h-5 text-blue-600" />
                    Manage Admins
                  </h2>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    Add or remove administrators for the invite code system.
                  </p>
                </div>
                <button
                  onClick={fetchAdmins}
                  disabled={loading}
                  className="p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
                  title="Refresh Admins"
                >
                  <RefreshCw className={`w-5 h-5 ${loading ? 'animate-spin' : ''}`} />
                </button>
              </div>

              <form onSubmit={handleAddAdmin} className="flex gap-2 max-w-md mb-8">
                <div className="relative flex-1">
                  <UserPlus className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 w-4 h-4" />
                  <input
                    type="text"
                    placeholder="Username to add"
                    className="w-full p-2.5 pl-9 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition-all text-sm"
                    value={newAdminUsername}
                    onChange={(e) => setNewAdminUsername(e.target.value)}
                  />
                </div>
                <button
                  type="submit"
                  disabled={loading || !newAdminUsername.trim()}
                  className="bg-blue-600 text-white px-4 py-2.5 rounded-lg font-medium hover:bg-blue-700 transition disabled:opacity-50 flex items-center gap-2 text-sm"
                >
                  Add Admin
                </button>
              </form>

              {newAdminPassword && (
                <div className="bg-green-50 dark:bg-green-900/20 border border-green-100 dark:border-green-900/30 p-4 rounded-lg mb-6 flex flex-col items-center gap-2">
                  <div className="flex items-center gap-2 text-green-700 dark:text-green-300 font-medium">
                    <Check className="w-5 h-5" />
                    Admin created successfully!
                  </div>
                  <div className="flex items-center gap-3 bg-white dark:bg-gray-700 px-4 py-2 rounded border border-green-200 dark:border-green-800 w-full justify-between">
                    <code className="text-lg font-mono text-gray-800 dark:text-white">
                      {newAdminPassword}
                    </code>
                    <button
                      onClick={() => copyToClipboard(newAdminPassword)}
                      className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-600 rounded transition"
                      title="Copy Password"
                    >
                      {copied === newAdminPassword ? (
                        <Check className="w-4 h-4 text-green-500" />
                      ) : (
                        <Copy className="w-4 h-4 text-gray-500" />
                      )}
                    </button>
                  </div>
                  <p className="text-xs text-green-600 dark:text-green-400">
                    Please save this password, it will not be shown again.
                  </p>
                </div>
              )}

              {error && (
                <div className="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-900/30 p-3 rounded-lg mb-6">
                  <p className="text-red-500 text-sm text-center">{error}</p>
                </div>
              )}

              <div className="overflow-x-auto">
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="border-b dark:border-gray-700">
                      <th className="py-4 px-4 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                        Username
                      </th>
                      <th className="py-4 px-4 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                        Added Date
                      </th>
                      <th className="py-4 px-4 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider text-right">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y dark:divide-gray-700">
                    {admins.length === 0 ? (
                      <tr>
                        <td
                          colSpan={3}
                          className="py-8 text-center text-gray-500 dark:text-gray-400"
                        >
                          No admins found.
                        </td>
                      </tr>
                    ) : (
                      admins.map((admin) => (
                        <tr
                          key={admin.username}
                          className="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition"
                        >
                          <td className="py-4 px-4">
                            <span className="font-medium text-gray-800 dark:text-white">
                              {admin.username}
                            </span>
                          </td>
                          <td className="py-4 px-4 text-sm text-gray-600 dark:text-gray-400">
                            {formatDate(admin.createdAt)}
                          </td>
                          <td className="py-4 px-4 text-right">
                            <button
                              onClick={() => handleRemoveAdmin(admin.username)}
                              className="p-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition"
                              title="Remove Admin"
                            >
                              <UserMinus className="w-5 h-5" />
                            </button>
                          </td>
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        )}

        {page === 'Home' && (
          <div className="space-y-6">
            {/* Controls */}
            <div className="bg-white dark:bg-gray-800 p-4 rounded-lg shadow-sm border dark:border-gray-700 flex flex-col sm:flex-row gap-4 items-stretch sm:items-center justify-between">
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min="1"
                  max="100"
                  className="w-full sm:w-20 p-2.5 sm:p-2 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded"
                  value={inviteCount}
                  onChange={(e) => setInviteCount(parseInt(e.target.value))}
                />
                <button
                  onClick={handleCreateInvites}
                  disabled={loading}
                  className="flex-1 sm:flex-none bg-blue-600 text-white px-4 py-2.5 sm:py-2 rounded flex items-center justify-center gap-2 hover:bg-blue-700 transition font-medium"
                >
                  <Plus className="w-4 h-4" /> Generate
                </button>
              </div>

              <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 sm:gap-4">
                <div className="flex items-center gap-2 relative">
                  <Filter className="w-4 h-4 text-gray-400 absolute left-3" />
                  <select
                    value={filter}
                    onChange={(e) => setFilter(e.target.value as FilterStatus)}
                    className="w-full sm:w-auto pl-10 pr-4 py-2.5 sm:py-2 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded appearance-none"
                  >
                    <option value="All">All Status</option>
                    <option value="Unused">Unused</option>
                    <option value="Used">Used</option>
                    <option value="Disabled">Disabled</option>
                  </select>
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={downloadCsv}
                    className="flex-1 sm:flex-none p-2.5 sm:p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded border dark:border-gray-600 flex items-center justify-center gap-2"
                  >
                    <Download className="w-4 h-4" /> Export CSV
                  </button>

                  <button
                    onClick={fetchInvites}
                    disabled={loading}
                    className="p-2.5 sm:p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded border dark:border-gray-600 min-w-[44px] flex items-center justify-center"
                  >
                    <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
                  </button>
                </div>
              </div>
            </div>

            {error && (
              <p className="text-red-500 bg-red-50 dark:bg-red-900/20 p-3 rounded border border-red-200 dark:border-red-900/30">
                {error}
              </p>
            )}

            {/* Desktop Table - Hidden on Mobile */}
            <div className="hidden md:block bg-white dark:bg-gray-800 rounded-lg shadow-sm border dark:border-gray-700 overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full text-left">
                  <thead className="bg-gray-50 dark:bg-gray-700/50 border-b dark:border-gray-700">
                    <tr>
                      <th className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                        Invite Code
                      </th>
                      <th className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                        Created At
                      </th>
                      <th className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                        Used By
                      </th>
                      <th className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                        DID
                      </th>
                      <th className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                        Used At
                      </th>
                      <th className="px-6 py-3 text-sm font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider text-right">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-100 dark:divide-gray-700">
                    {filteredInvites.map((invite) => {
                      const status = getStatus(invite);
                      const usedAt = getUsedAt(invite);
                      const usedBy = getUsedBy(invite);
                      const resolvedHandle = usedBy ? handles[usedBy] : null;
                      return (
                        <tr
                          key={invite.code}
                          className="hover:bg-gray-50 dark:hover:bg-gray-700/30 transition"
                        >
                          <td className="px-6 py-4 font-mono text-sm flex items-center gap-2 dark:text-gray-200">
                            {invite.code}
                            <button
                              onClick={() => copyToClipboard(invite.code)}
                              className="text-gray-400 hover:text-blue-600 p-1"
                            >
                              {copied === invite.code ? (
                                <Check className="w-4 h-4 text-green-500" />
                              ) : (
                                <Copy className="w-4 h-4" />
                              )}
                            </button>
                          </td>
                          <td className="px-6 py-4">
                            <span
                              className={`px-2 py-1 rounded-full text-xs font-medium ${
                                status === 'Unused'
                                  ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                                  : status === 'Used'
                                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
                                    : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
                              }`}
                            >
                              {status}
                            </span>
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                            {formatDate(invite.createdAt)}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                            {resolvedHandle ? (
                              <a
                                href={`https://bsky.app/profile/${resolvedHandle}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-blue-600 hover:underline"
                              >
                                {resolvedHandle}
                              </a>
                            ) : (
                              '-'
                            )}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                            {usedBy ? (
                              <span className="font-mono text-xs" title={usedBy}>
                                {usedBy}
                              </span>
                            ) : (
                              '-'
                            )}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                            {formatDate(usedAt)}
                          </td>
                          <td className="px-6 py-4 text-right">
                            {status === 'Unused' && (
                              <button
                                onClick={() => handleDisableInvite(invite.code)}
                                className="text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 p-2 rounded transition"
                                title="Disable"
                              >
                                <Trash2 className="w-4 h-4" />
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

            {/* Mobile Cards - Shown on Mobile */}
            <div className="md:hidden space-y-4">
              {filteredInvites.map((invite) => {
                const status = getStatus(invite);
                const usedAt = getUsedAt(invite);
                const usedBy = getUsedBy(invite);
                const resolvedHandle = usedBy ? handles[usedBy] : null;
                return (
                  <div
                    key={invite.code}
                    className="bg-white dark:bg-gray-800 p-4 rounded-lg shadow-sm border dark:border-gray-700 space-y-3"
                  >
                    <div className="flex justify-between items-center">
                      <div className="font-mono text-lg font-bold dark:text-white flex items-center gap-2">
                        {invite.code}
                        <button
                          onClick={() => copyToClipboard(invite.code)}
                          className="text-gray-400 hover:text-blue-600 p-2"
                        >
                          {copied === invite.code ? (
                            <Check className="w-5 h-5 text-green-500" />
                          ) : (
                            <Copy className="w-5 h-5" />
                          )}
                        </button>
                      </div>
                      <span
                        className={`px-2 py-1 rounded-full text-xs font-medium ${
                          status === 'Unused'
                            ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                            : status === 'Used'
                              ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
                              : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
                        }`}
                      >
                        {status}
                      </span>
                    </div>
                    <div className="grid grid-cols-2 gap-2 text-sm">
                      <div className="col-span-1">
                        <p className="text-gray-500 dark:text-gray-400 text-xs uppercase font-semibold">
                          Created At
                        </p>
                        <p className="dark:text-gray-300">{formatDate(invite.createdAt)}</p>
                      </div>
                      <div className="col-span-1 text-right">
                        <p className="text-gray-500 dark:text-gray-400 text-xs uppercase font-semibold">
                          Used At
                        </p>
                        <p className="dark:text-gray-300">{formatDate(usedAt)}</p>
                      </div>
                      <div className="col-span-2">
                        <p className="text-gray-500 dark:text-gray-400 text-xs uppercase font-semibold">
                          Used By
                        </p>
                        <p className="dark:text-gray-300 truncate">
                          {resolvedHandle ? (
                            <a
                              href={`https://bsky.app/profile/${resolvedHandle}`}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-blue-600 hover:underline"
                            >
                              {resolvedHandle}
                            </a>
                          ) : (
                            '-'
                          )}
                        </p>
                      </div>
                      <div className="col-span-2">
                        <p className="text-gray-500 dark:text-gray-400 text-xs uppercase font-semibold">
                          DID
                        </p>
                        <p className="dark:text-gray-300 font-mono text-xs truncate" title={usedBy}>
                          {usedBy || '-'}
                        </p>
                      </div>
                    </div>
                    {status === 'Unused' && (
                      <div className="pt-2 border-t dark:border-gray-700">
                        <button
                          onClick={() => handleDisableInvite(invite.code)}
                          className="w-full flex items-center justify-center gap-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 py-2 rounded transition border border-red-100 dark:border-red-900/30"
                        >
                          <Trash2 className="w-4 h-4" /> Disable Invite
                        </button>
                      </div>
                    )}
                  </div>
                );
              })}
              {filteredInvites.length === 0 && (
                <div className="text-center py-8 bg-white dark:bg-gray-800 rounded-lg border dark:border-gray-700">
                  <p className="text-gray-500 dark:text-gray-400">No invite codes found</p>
                </div>
              )}
            </div>
          </div>
        )}

        {page === 'QrVerify' && (
          <div className="max-w-md mx-auto space-y-6 text-center px-2">
            <h2 className="text-xl font-bold dark:text-white">Setup Multi-Factor Authentication</h2>
            {qrCode && (
              <div className="bg-white p-4 rounded-xl shadow-md inline-block">
                <img src={qrCode} alt="OTP QR Code" className="mx-auto w-48 h-48 md:w-64 md:h-64" />
              </div>
            )}
            <p className="text-gray-600 dark:text-gray-400 text-sm md:text-base">
              Scan this QR code with your authenticator app, then enter the code below to verify.
            </p>
            <div className="space-y-4">
              <input
                type="text"
                inputMode="numeric"
                pattern="[0-9]*"
                placeholder="000000"
                className="w-full p-4 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-xl text-center text-3xl tracking-[0.5em] focus:ring-2 focus:ring-blue-500 outline-none"
                value={otpToken}
                onChange={(e) => setOtpToken(e.target.value)}
              />
              {error && <p className="text-red-500 text-sm mb-4">{error}</p>}
              <div className="flex flex-col sm:flex-row gap-3">
                <button
                  onClick={() => {
                    setError(null);
                    setPage('Home');
                  }}
                  className="flex-1 p-4 border dark:border-gray-600 dark:text-white rounded-xl font-bold hover:bg-gray-100 dark:hover:bg-gray-700 transition"
                >
                  Cancel
                </button>
                <button
                  onClick={handleVerifyOtp}
                  disabled={loading}
                  className="flex-1 bg-blue-600 text-white p-4 rounded-xl font-bold hover:bg-blue-700 transition shadow-lg shadow-blue-500/20 disabled:opacity-50"
                >
                  {loading ? 'Verifying...' : 'Verify'}
                </button>
              </div>
            </div>
          </div>
        )}

        {page === 'QrValidate' && (
          <div className="max-w-md mx-auto space-y-6 text-center px-2">
            <div className="bg-blue-600 p-4 rounded-2xl mx-auto w-fit shadow-lg shadow-blue-500/20">
              <ShieldCheck className="text-white w-10 h-10" />
            </div>
            <h2 className="text-2xl font-bold dark:text-white">Two-Factor Authentication</h2>
            <p className="text-gray-600 dark:text-gray-400">
              Please enter the 6-digit code from your authenticator app.
            </p>
            <div className="space-y-4">
              <input
                type="text"
                inputMode="numeric"
                pattern="[0-9]*"
                placeholder="000000"
                className="w-full p-4 border dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-xl text-center text-3xl tracking-[0.5em] focus:ring-2 focus:ring-blue-500 outline-none"
                value={otpToken}
                onChange={(e) => setOtpToken(e.target.value)}
                autoFocus
              />
              {error && <p className="text-red-500 text-sm mb-4">{error}</p>}
              <div className="flex flex-col sm:flex-row gap-3">
                <button
                  onClick={() => {
                    setError(null);
                    if (token) {
                      setPage('Home');
                    } else {
                      setPage('Login');
                    }
                  }}
                  className="flex-1 p-4 border dark:border-gray-600 dark:text-white rounded-xl font-bold hover:bg-gray-100 dark:hover:bg-gray-700 transition"
                >
                  Cancel
                </button>
                <button
                  onClick={handleValidateOtp}
                  disabled={loading}
                  className="flex-1 bg-blue-600 text-white p-4 rounded-xl font-bold hover:bg-blue-700 transition shadow-lg shadow-blue-500/20 disabled:opacity-50"
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
