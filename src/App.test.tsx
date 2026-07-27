import { describe, it, expect, beforeAll, afterEach, afterAll } from 'vitest';
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
  http.get(`${API_HOST}/api/account/emails`, () => HttpResponse.json({ emails: {} }))
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
