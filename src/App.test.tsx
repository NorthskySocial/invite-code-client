import { describe, it, expect, beforeAll, afterEach, afterAll, vi } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import App from './App';

const API_HOST = 'https://frontend.myapp.local';

const invites = [
  {
    code: 'UNUSED-CODE',
    available: 1,
    disabled: false,
    forAccount: 'admin',
    createdBy: 'admin',
    createdAt: '2026-01-25T08:02:05.614Z',
    uses: [],
  },
  {
    code: 'USED-CODE',
    available: 0,
    disabled: false,
    forAccount: 'admin',
    createdBy: 'admin',
    createdAt: '2026-01-25T08:02:05.614Z',
    uses: [{ usedBy: 'user1', usedAt: '2026-01-25T08:12:55.280Z' }],
  },
  {
    code: 'DISABLED-CODE',
    available: 1,
    disabled: true,
    forAccount: 'admin',
    createdBy: 'admin',
    createdAt: '2026-01-25T08:02:05.614Z',
    uses: [],
  },
];

const server = setupServer(
  http.get(`${API_HOST}/api/invite-codes`, () => HttpResponse.json({ codes: invites })),
  http.get(`${API_HOST}/api/admins`, () => HttpResponse.json({ admins: [] })),
  http.post(`${API_HOST}/api/create-invite-codes`, () => HttpResponse.json({ success: true })),
  http.post(`${API_HOST}/api/disable-invite-codes`, () => HttpResponse.json({ success: true })),
  http.post(`${API_HOST}/api/admins`, () =>
    HttpResponse.json({ status: 'success', message: 'ok', password: 'generated-pass' })
  ),
  http.delete(`${API_HOST}/api/admins/:username`, () => HttpResponse.json({ success: true })),
  http.post(`${API_HOST}/api/auth/otp/verify`, () => HttpResponse.json({ token: 'otp-token' })),
  http.post(`${API_HOST}/api/auth/otp/validate`, () => HttpResponse.json({ token: 'otp-token' }))
);

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const renderLoggedIn = () => {
  localStorage.setItem('token', 'fake-token');
  return render(<App />);
};

// Invite codes render in both a desktop table and a mobile card list.
const expectCodeVisible = async (code: string) => {
  await waitFor(() => expect(screen.getAllByText(code).length).toBeGreaterThan(0));
};

describe('App', () => {
  it('fetches and renders invites on mount when a token is present', async () => {
    renderLoggedIn();

    await expectCodeVisible('UNUSED-CODE');
    await expectCodeVisible('USED-CODE');
    await expectCodeVisible('DISABLED-CODE');
  });

  it('shows the login screen when no token is stored and does not fetch', async () => {
    render(<App />);

    expect(screen.getByPlaceholderText('Username')).toBeInTheDocument();
    expect(screen.queryByText('UNUSED-CODE')).not.toBeInTheDocument();
  });

  it('filters the invite list by status', async () => {
    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');

    const filterSelect = screen.getByRole('combobox');

    fireEvent.change(filterSelect, { target: { value: 'Unused' } });
    await waitFor(() => expect(screen.queryByText('USED-CODE')).not.toBeInTheDocument());
    expect(screen.getAllByText('UNUSED-CODE').length).toBeGreaterThan(0);
    expect(screen.queryByText('DISABLED-CODE')).not.toBeInTheDocument();

    fireEvent.change(filterSelect, { target: { value: 'Used' } });
    await waitFor(() => expect(screen.getAllByText('USED-CODE').length).toBeGreaterThan(0));
    expect(screen.queryByText('UNUSED-CODE')).not.toBeInTheDocument();

    fireEvent.change(filterSelect, { target: { value: 'Disabled' } });
    await waitFor(() => expect(screen.getAllByText('DISABLED-CODE').length).toBeGreaterThan(0));
    expect(screen.queryByText('USED-CODE')).not.toBeInTheDocument();

    fireEvent.change(filterSelect, { target: { value: 'All' } });
    await expectCodeVisible('USED-CODE');
    await expectCodeVisible('UNUSED-CODE');
    await expectCodeVisible('DISABLED-CODE');
  });

  it('logs out when fetching invites returns 401', async () => {
    server.use(
      http.get(`${API_HOST}/api/invite-codes`, () => new HttpResponse(null, { status: 401 }))
    );

    renderLoggedIn();

    await waitFor(() => expect(screen.getByPlaceholderText('Username')).toBeInTheDocument());
    expect(localStorage.getItem('token')).toBeNull();
  });

  it('clears the session when the logout button is clicked', async () => {
    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');

    fireEvent.click(screen.getByTitle('Logout'));

    await waitFor(() => expect(screen.getByPlaceholderText('Username')).toBeInTheDocument());
    expect(localStorage.getItem('token')).toBeNull();
  });

  it('fetches invites once on mount and does not refetch when filtering', async () => {
    let inviteRequests = 0;
    server.use(
      http.get(`${API_HOST}/api/invite-codes`, () => {
        inviteRequests += 1;
        return HttpResponse.json({ codes: invites });
      })
    );

    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');
    await waitFor(() => expect(inviteRequests).toBe(1));

    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'Used' } });
    await waitFor(() => expect(screen.queryByText('UNUSED-CODE')).not.toBeInTheDocument());

    expect(inviteRequests).toBe(1);
  });
});

const fillLogin = (username: string) => {
  fireEvent.change(screen.getByPlaceholderText('Username'), { target: { value: username } });
  fireEvent.change(screen.getByPlaceholderText('Password'), { target: { value: 'pw' } });
  fireEvent.click(screen.getByRole('button', { name: 'Login' }));
};

describe('login flow', () => {
  it('stores the token and shows Home on a successful login', async () => {
    server.use(
      http.post(`${API_HOST}/api/auth/login`, () =>
        HttpResponse.json({
          token: 'new-token',
          otp_enabled: true,
          otp_verified: true,
          username: 'admin',
        })
      )
    );

    render(<App />);
    fillLogin('admin');

    await waitFor(() => expect(localStorage.getItem('token')).toBe('new-token'));
    await expectCodeVisible('UNUSED-CODE');
  });

  it('routes to the OTP validation screen when 2FA is required', async () => {
    server.use(
      http.post(`${API_HOST}/api/auth/login`, () =>
        HttpResponse.json({ otp_enabled: true, otp_verified: true })
      )
    );

    render(<App />);
    fillLogin('twofa');

    await waitFor(() =>
      expect(screen.getByText('Two-Factor Authentication')).toBeInTheDocument()
    );
    expect(localStorage.getItem('token')).toBeNull();
  });

  it('routes to the OTP setup screen when an auth URL is returned', async () => {
    server.use(
      http.post(`${API_HOST}/api/auth/login`, () =>
        HttpResponse.json({
          otp_enabled: false,
          otp_verified: false,
          otp_auth_url: 'otpauth://totp/InviteCode:x?secret=S',
        })
      )
    );

    render(<App />);
    fillLogin('newuser');

    await waitFor(() =>
      expect(screen.getByText('Setup Multi-Factor Authentication')).toBeInTheDocument()
    );
    expect(screen.getByAltText('OTP QR Code')).toBeInTheDocument();
  });

  it('shows an error message when login fails', async () => {
    server.use(
      http.post(`${API_HOST}/api/auth/login`, () => new HttpResponse(null, { status: 401 }))
    );

    render(<App />);
    fillLogin('bad');

    await waitFor(() => expect(screen.getByText('Login failed')).toBeInTheDocument());
  });
});

describe('invite actions', () => {
  it('creates invites and refetches the list', async () => {
    let created = 0;
    server.use(
      http.post(`${API_HOST}/api/create-invite-codes`, () => {
        created += 1;
        return HttpResponse.json({ success: true });
      })
    );

    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');

    fireEvent.click(screen.getByRole('button', { name: /Generate/ }));

    await waitFor(() => expect(created).toBe(1));
  });

  it('disables an unused invite with the correct code', async () => {
    let disabledCode = '';
    server.use(
      http.post(`${API_HOST}/api/disable-invite-codes`, async ({ request }) => {
        const body = (await request.json()) as { codes: string[] };
        disabledCode = body.codes[0];
        return HttpResponse.json({ success: true });
      })
    );

    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');

    fireEvent.click(screen.getAllByTitle('Disable')[0]);

    await waitFor(() => expect(disabledCode).toBe('UNUSED-CODE'));
  });

  it('shows an error when creating invites fails', async () => {
    server.use(
      http.post(`${API_HOST}/api/create-invite-codes`, () =>
        HttpResponse.json({ error: 'nope' }, { status: 500 })
      )
    );

    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');

    fireEvent.click(screen.getByRole('button', { name: /Generate/ }));

    await waitFor(() => expect(screen.getByText('nope')).toBeInTheDocument());
  });
});

describe('admins page', () => {
  const goToAdmins = async () => {
    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');
    fireEvent.click(screen.getByRole('button', { name: 'Admins' }));
  };

  it('adds an admin and reveals the generated password', async () => {
    await goToAdmins();
    await waitFor(() => expect(screen.getByText('No admins found.')).toBeInTheDocument());

    fireEvent.change(screen.getByPlaceholderText('Username to add'), {
      target: { value: 'bob' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Add Admin' }));

    await waitFor(() => expect(screen.getByText('generated-pass')).toBeInTheDocument());
  });

  it('removes an admin after confirmation', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    let deleted = '';
    server.use(
      http.get(`${API_HOST}/api/admins`, () =>
        HttpResponse.json({
          admins: [{ username: 'carol', createdAt: '2026-01-25T08:02:05.614Z' }],
        })
      ),
      http.delete(`${API_HOST}/api/admins/:username`, ({ params }) => {
        deleted = params.username as string;
        return HttpResponse.json({ success: true });
      })
    );

    await goToAdmins();
    await waitFor(() => expect(screen.getByText('carol')).toBeInTheDocument());

    fireEvent.click(screen.getByTitle('Remove Admin'));

    await waitFor(() => expect(deleted).toBe('carol'));
    confirmSpy.mockRestore();
  });

  it('does not remove an admin when confirmation is cancelled', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    let deleteCalled = false;
    server.use(
      http.get(`${API_HOST}/api/admins`, () =>
        HttpResponse.json({
          admins: [{ username: 'carol', createdAt: '2026-01-25T08:02:05.614Z' }],
        })
      ),
      http.delete(`${API_HOST}/api/admins/:username`, () => {
        deleteCalled = true;
        return HttpResponse.json({ success: true });
      })
    );

    await goToAdmins();
    await waitFor(() => expect(screen.getByText('carol')).toBeInTheDocument());

    fireEvent.click(screen.getByTitle('Remove Admin'));

    expect(deleteCalled).toBe(false);
    confirmSpy.mockRestore();
  });

  it('copies the generated password to the clipboard', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    });

    await goToAdmins();
    await waitFor(() => expect(screen.getByText('No admins found.')).toBeInTheDocument());
    fireEvent.change(screen.getByPlaceholderText('Username to add'), {
      target: { value: 'bob' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Add Admin' }));
    await waitFor(() => expect(screen.getByText('generated-pass')).toBeInTheDocument());

    fireEvent.click(screen.getByTitle('Copy Password'));

    expect(writeText).toHaveBeenCalledWith('generated-pass');
  });
});

describe('OTP verification and validation', () => {
  it('verifies an OTP setup token and lands on Home', async () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});
    server.use(
      http.post(`${API_HOST}/api/auth/login`, () =>
        HttpResponse.json({
          otp_enabled: false,
          otp_verified: false,
          otp_auth_url: 'otpauth://totp/x?secret=S',
        })
      )
    );

    render(<App />);
    fillLogin('newuser');
    await waitFor(() =>
      expect(screen.getByText('Setup Multi-Factor Authentication')).toBeInTheDocument()
    );

    fireEvent.change(screen.getByPlaceholderText('000000'), { target: { value: '123456' } });
    fireEvent.click(screen.getByRole('button', { name: 'Verify' }));

    await waitFor(() => expect(localStorage.getItem('token')).toBe('otp-token'));
    alertSpy.mockRestore();
  });

  it('validates a 2FA token and lands on Home', async () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});
    server.use(
      http.post(`${API_HOST}/api/auth/login`, () =>
        HttpResponse.json({ otp_enabled: true, otp_verified: true })
      )
    );

    render(<App />);
    fillLogin('twofa');
    await waitFor(() =>
      expect(screen.getByText('Two-Factor Authentication')).toBeInTheDocument()
    );

    fireEvent.change(screen.getByPlaceholderText('000000'), { target: { value: '123456' } });
    fireEvent.click(screen.getByRole('button', { name: 'Validate' }));

    await waitFor(() => expect(localStorage.getItem('token')).toBe('otp-token'));
    alertSpy.mockRestore();
  });
});

describe('DID handle resolution', () => {
  const didInvites = [
    {
      code: 'DID-CODE',
      available: 0,
      disabled: false,
      forAccount: 'admin',
      createdBy: 'admin',
      createdAt: '2026-01-25T08:02:05.614Z',
      uses: [{ usedBy: 'did:plc:abc', usedAt: '2026-01-25T08:12:55.280Z' }],
    },
  ];

  it('resolves a DID to a handle and renders a profile link', async () => {
    server.use(
      http.get(`${API_HOST}/api/invite-codes`, () => HttpResponse.json({ codes: didInvites })),
      http.get('https://plc.directory/:did', () =>
        HttpResponse.json({ alsoKnownAs: ['at://alice.bsky.social'] })
      )
    );

    renderLoggedIn();

    await waitFor(() =>
      expect(screen.getAllByText('alice.bsky.social').length).toBeGreaterThan(0)
    );
    const link = screen.getAllByRole('link', { name: 'alice.bsky.social' })[0];
    expect(link).toHaveAttribute('href', 'https://bsky.app/profile/alice.bsky.social');
  });

  it('resolves each unique DID only once', async () => {
    let plcRequests = 0;
    const twoUses = [
      { ...didInvites[0], code: 'DID-1' },
      { ...didInvites[0], code: 'DID-2' },
    ];
    server.use(
      http.get(`${API_HOST}/api/invite-codes`, () => HttpResponse.json({ codes: twoUses })),
      http.get('https://plc.directory/:did', () => {
        plcRequests += 1;
        return HttpResponse.json({ alsoKnownAs: ['at://alice.bsky.social'] });
      })
    );

    renderLoggedIn();

    await waitFor(() =>
      expect(screen.getAllByText('alice.bsky.social').length).toBeGreaterThan(0)
    );
    expect(plcRequests).toBe(1);
  });

  it('logs an error and falls back to the DID when resolution fails', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    server.use(
      http.get(`${API_HOST}/api/invite-codes`, () => HttpResponse.json({ codes: didInvites })),
      http.get('https://plc.directory/:did', () => new HttpResponse(null, { status: 500 }))
    );

    renderLoggedIn();

    await waitFor(() => expect(errorSpy).toHaveBeenCalled());
    errorSpy.mockRestore();
  });
});

describe('CSV export', () => {
  it('builds a CSV blob with headers and invite rows', async () => {
    let capturedBlob: Blob | null = null;
    const originalCreate = URL.createObjectURL;
    const originalRevoke = URL.revokeObjectURL;
    URL.createObjectURL = vi.fn((blob: Blob) => {
      capturedBlob = blob;
      return 'blob:mock';
    });
    URL.revokeObjectURL = vi.fn();
    const clickSpy = vi
      .spyOn(HTMLAnchorElement.prototype, 'click')
      .mockImplementation(() => {});

    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');

    fireEvent.click(screen.getByRole('button', { name: /Export CSV/ }));

    expect(capturedBlob).not.toBeNull();
    const text = await new Promise<string>((resolve) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.readAsText(capturedBlob!);
    });
    expect(text).toContain('Invite Code,Status,Created At,Used By,Account Status,Email,Used At');
    expect(text).toContain('UNUSED-CODE');
    expect(text).toContain('USED-CODE');

    URL.createObjectURL = originalCreate;
    URL.revokeObjectURL = originalRevoke;
    clickSpy.mockRestore();
  });
});

describe('theme and demo mode', () => {
  it('toggles dark mode and persists the choice', async () => {
    renderLoggedIn();
    await expectCodeVisible('UNUSED-CODE');

    expect(document.documentElement.classList.contains('dark')).toBe(false);

    fireEvent.click(screen.getByTitle('Toggle Theme'));

    await waitFor(() =>
      expect(document.documentElement.classList.contains('dark')).toBe(true)
    );
    expect(localStorage.getItem('theme')).toBe('dark');
  });

  it('runs against the mock backend when demo mode is enabled', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('switch'));
    fireEvent.click(screen.getByRole('button', { name: /Start Demo/ }));

    await waitFor(
      () => expect(screen.getAllByText('DEMO-123').length).toBeGreaterThan(0),
      { timeout: 3000 }
    );
    expect(localStorage.getItem('demo_mode')).toBe('true');
  });
});

