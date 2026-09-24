import { describe, it, expect, beforeAll, afterEach, afterAll } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { apiService, api, updateApiBaseURL, getBaseURL, mockApiService } from './api';

const handlers = [
  http.post('https://frontend.myapp.local/api/auth/login', async ({ request }) => {
    const { username } = (await request.json()) as { username: string };
    if (username === 'testuser') {
      return HttpResponse.json({
        username: 'testuser',
        otp_enabled: true,
        otp_verified: true,
        otp_auth_url: null,
      });
    }
    return new HttpResponse(null, { status: 401 });
  }),

  http.post('https://frontend.myapp.local/api/auth/logout', () =>
    new HttpResponse(null, { status: 204 })
  ),

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
          uses: [
            {
              usedBy: 'user1',
              usedByHandle: null,
              usedByEmail: null,
              usedAt: '2026-01-25T08:12:55.280Z',
              usedByDeactivated: null,
            },
          ],
        },
      ],
      cursor: null,
    });
  }),

  http.post('https://frontend.myapp.local/api/create-invite-codes', () =>
    new HttpResponse(null, { status: 200 })
  ),

  http.post('https://frontend.myapp.local/api/disable-invite-codes', () =>
    new HttpResponse(null, { status: 200 })
  ),

  http.post('https://frontend.myapp.local/api/auth/otp/validate', () => {
    return HttpResponse.json({ otp_valid: true });
  }),

  http.post('https://frontend.myapp.local/api/auth/otp/verify', () => {
    return HttpResponse.json({
      otp_verified: true,
      user: {
        username: 'testuser',
        otp_enabled: true,
        otp_verified: true,
        otp_auth_url: null,
      },
    });
  }),

  http.get('https://frontend.myapp.local/api/admins', () => {
    return HttpResponse.json({
      status: 'success',
      admins: [{ username: 'admin', otp_enabled: true, otp_verified: true }],
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

  http.delete('https://frontend.myapp.local/api/admins', () => {
    return HttpResponse.json({ status: 'success', message: 'Admin user removed successfully' });
  }),
];

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('apiService', () => {
  it('login should return admin state and username', async () => {
    const response = await apiService.login('testuser', 'password');
    expect(response.data.username).toBe('testuser');
    expect(response.data.otp_verified).toBe(true);
  });

  it('logout should invalidate the session', async () => {
    const response = await apiService.logout();
    expect(response.status).toBe(204);
  });

  it('getInviteCodes should return list of invite codes', async () => {
    const response = await apiService.getInviteCodes();
    expect(response.data.codes).toHaveLength(2);
    expect(response.data.codes[0].code).toBe('CODE1');
    expect(response.data.cursor).toBeNull();
  });

  it('createInviteCodes should send correct count', async () => {
    const response = await apiService.createInviteCodes(5);
    expect(response.data).toBe('');
  });

  it('disableInviteCode should send correct code', async () => {
    const response = await apiService.disableInviteCode('CODE1');
    expect(response.data).toBe('');
  });

  it('validateOtp should return success and include credentials', async () => {
    let credentialsIncluded = false;
    server.use(
      http.post('https://frontend.myapp.local/api/auth/otp/validate', ({ request }) => {
        credentialsIncluded = request.credentials === 'include';
        return HttpResponse.json({ otp_valid: true });
      })
    );
    const response = await apiService.validateOtp('123456');
    expect(response.data).toEqual({ otp_valid: true });
    expect(credentialsIncluded).toBe(true);
  });

  it('verifyOtp should return success', async () => {
    const response = await apiService.verifyOtp('123456');
    expect(response.data.otp_verified).toBe(true);
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

  it('removeAdmin should send the username in the request body', async () => {
    let requestedUrl = '';
    let requestedBody: { username?: string } = {};
    server.use(
      http.delete('https://frontend.myapp.local/api/admins', async ({ request }) => {
        requestedUrl = request.url;
        requestedBody = (await request.json()) as { username?: string };
        return HttpResponse.json({ status: 'success', message: 'Admin user removed successfully' });
      })
    );
    const response = await apiService.removeAdmin('gone');
    expect(response.data.status).toBe('success');
    expect(requestedUrl).toBe('https://frontend.myapp.local/api/admins');
    expect(requestedBody).toEqual({ username: 'gone' });
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
  it('login returns verified admin state for a regular user', async () => {
    const response = await mockApiService.login('demo-user', 'pw');
    expect(response.data.username).toBe('demo-user');
    expect(response.data.otp_verified).toBe(true);
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
