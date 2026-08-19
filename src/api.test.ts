import { describe, it, expect, beforeAll, afterEach, afterAll } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { apiService, api, updateApiBaseURL, getBaseURL, mockApiService } from './api';

const handlers = [
  http.post('https://frontend.myapp.local/api/auth/login', async ({ request }) => {
    const { username } = (await request.json()) as { username: string };
    if (username === 'testuser') {
      return HttpResponse.json({ token: 'fake-token', username: 'testuser' });
    }
    return new HttpResponse(null, { status: 401 });
  }),

  http.get('https://frontend.myapp.local/api/invite-codes', () => {
    return HttpResponse.json({
      codes: [
        {
          code: 'CODE1',
          available: 1,
          disabled: false,
          forAccount: 'admin',
          createdBy: 'admin',
          createdAt: '2026-01-25T08:02:05.614Z',
          uses: [],
        },
        {
          code: 'CODE2',
          available: 0,
          disabled: false,
          forAccount: 'admin',
          createdBy: 'admin',
          createdAt: '2026-01-25T08:02:05.614Z',
          uses: [{ usedBy: 'user1', usedAt: '2026-01-25T08:12:55.280Z' }],
        },
      ],
    });
  }),

  http.post('https://frontend.myapp.local/api/create-invite-codes', async ({ request }) => {
    const { codeCount } = (await request.json()) as { codeCount: number };
    return HttpResponse.json({ message: `Created ${codeCount} codes` });
  }),

  http.post('https://frontend.myapp.local/api/disable-invite-codes', async ({ request }) => {
    const { codes } = (await request.json()) as { codes: string[] };
    return HttpResponse.json({ message: `Disabled code ${codes[0]}` });
  }),

  http.get('https://frontend.myapp.local/api/auth/otp/generate', () => {
    return HttpResponse.json({ qr_code: 'fake-qr-code' });
  }),

  http.post('https://frontend.myapp.local/api/auth/otp/validate', () => {
    return HttpResponse.json({ success: true });
  }),

  http.post('https://frontend.myapp.local/api/auth/otp/verify', () => {
    return HttpResponse.json({ success: true });
  }),

  http.get('https://frontend.myapp.local/api/admins', () => {
    return HttpResponse.json({
      admins: [{ username: 'admin', createdAt: '2026-01-25T08:02:05.614Z' }],
    });
  }),

  http.post('https://frontend.myapp.local/api/admins', async ({ request }) => {
    const { username } = (await request.json()) as { username: string };
    return HttpResponse.json({
      status: 'success',
      message: 'Admin user created successfully',
      password: `pw-for-${username}`,
    });
  }),

  http.delete('https://frontend.myapp.local/api/admins/:username', () => {
    return HttpResponse.json({ success: true });
  }),
];

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('apiService', () => {
  it('login should return token and username', async () => {
    const response = await apiService.login('testuser', 'password');
    expect(response.data).toEqual({ token: 'fake-token', username: 'testuser' });
  });

  it('getInviteCodes should return list of invite codes', async () => {
    const response = await apiService.getInviteCodes();
    expect(response.data.codes).toHaveLength(2);
    expect(response.data.codes[0].code).toBe('CODE1');
  });

  it('createInviteCodes should send correct count', async () => {
    const response = await apiService.createInviteCodes(5);
    expect(response.data).toEqual({ message: 'Created 5 codes' });
  });

  it('disableInviteCode should send correct code', async () => {
    const response = await apiService.disableInviteCode('CODE1');
    expect(response.data).toEqual({ message: 'Disabled code CODE1' });
  });

  it('generateOtp should return qr_code', async () => {
    const response = await apiService.generateOtp();
    expect(response.data).toEqual({ qr_code: 'fake-qr-code' });
  });

  it('validateOtp should return success and include credentials', async () => {
    let credentialsIncluded = false;
    server.use(
      http.post('https://frontend.myapp.local/api/auth/otp/validate', ({ request }) => {
        credentialsIncluded = request.credentials === 'include';
        return HttpResponse.json({ success: true });
      })
    );
    const response = await apiService.validateOtp('123456');
    expect(response.data).toEqual({ success: true });
    expect(credentialsIncluded).toBe(true);
  });

  it('verifyOtp should return success', async () => {
    const response = await apiService.verifyOtp('123456');
    expect(response.data).toEqual({ success: true });
  });

  it('getAdmins should return list of admins', async () => {
    const response = await apiService.getAdmins();
    expect(response.data.admins).toHaveLength(1);
    expect(response.data.admins[0].username).toBe('admin');
  });

  it('addAdmin should send the username and return a generated password', async () => {
    const response = await apiService.addAdmin('newadmin');
    expect(response.data.status).toBe('success');
    expect(response.data.password).toBe('pw-for-newadmin');
  });

  it('removeAdmin should target the username in the URL', async () => {
    let requestedUrl = '';
    server.use(
      http.delete('https://frontend.myapp.local/api/admins/:username', ({ request }) => {
        requestedUrl = request.url;
        return HttpResponse.json({ success: true });
      })
    );
    const response = await apiService.removeAdmin('gone');
    expect(response.data).toEqual({ success: true });
    expect(requestedUrl).toContain('/api/admins/gone');
  });
});

describe('auth token interceptor', () => {
  it('adds an Authorization header when a token is stored', async () => {
    localStorage.setItem('token', 'abc123');
    let authHeader: string | null = null;
    server.use(
      http.get('https://frontend.myapp.local/api/invite-codes', ({ request }) => {
        authHeader = request.headers.get('Authorization');
        return HttpResponse.json({ codes: [] });
      })
    );
    await apiService.getInviteCodes();
    expect(authHeader).toBe('Bearer abc123');
  });

  it('omits the Authorization header when no token is stored', async () => {
    let authHeader: string | null = 'unset';
    server.use(
      http.get('https://frontend.myapp.local/api/invite-codes', ({ request }) => {
        authHeader = request.headers.get('Authorization');
        return HttpResponse.json({ codes: [] });
      })
    );
    await apiService.getInviteCodes();
    expect(authHeader).toBeNull();
  });
});

describe('base URL resolution', () => {
  it('getBaseURL falls back to the default host when nothing is configured', () => {
    expect(getBaseURL()).toBe('https://frontend.myapp.local/');
  });

  it('getBaseURL prefers the stored api_host', () => {
    localStorage.setItem('api_host', 'https://custom.example.com/');
    expect(getBaseURL()).toBe('https://custom.example.com/');
  });

  it('updateApiBaseURL changes the axios default base URL', () => {
    const original = api.defaults.baseURL;
    try {
      updateApiBaseURL('https://changed.example.com/');
      expect(api.defaults.baseURL).toBe('https://changed.example.com/');
    } finally {
      api.defaults.baseURL = original;
    }
  });
});

describe('mockApiService', () => {
  it('login returns a demo token for a regular user', async () => {
    const response = await mockApiService.login('demo-user', 'pw');
    expect(response.data.token).toBe('mock-token');
    expect(response.data.username).toBe('demo-user');
  });

  it('login returns OTP setup fields for a new user', async () => {
    const response = await mockApiService.login('new-user', 'pw');
    expect(response.data.otp_enabled).toBe(false);
    expect(response.data.otp_auth_url).toContain('otpauth://');
  });

  it('getInviteCodes returns the demo fixtures', async () => {
    const response = await mockApiService.getInviteCodes();
    expect(response.data.codes).toHaveLength(4);
    expect(response.data.codes.map((c) => c.code)).toContain('DEMO-123');
  });

  it('addAdmin persists the new admin and returns a password', async () => {
    const response = await mockApiService.addAdmin('grace');
    expect(response.data.status).toBe('success');
    expect(response.data.password).toContain('mock-generated-password-');

    const after = await mockApiService.getAdmins();
    expect(after.data.admins.map((a) => a.username)).toContain('grace');
  });

  it('removeAdmin drops the admin from the persisted list', async () => {
    await mockApiService.removeAdmin('admin');
    const after = await mockApiService.getAdmins();
    expect(after.data.admins.map((a) => a.username)).not.toContain('admin');
  });
});
