import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { UsageBar } from '../components/UsageBar';

describe('UsageBar', () => {
  it('renders with none warning level', () => {
    render(<UsageBar ratio={0.5} warningLevel="none" />);
    const bar = document.querySelector('.bg-\\[\\#10B981\\]');
    expect(bar).toBeInTheDocument();
  });

  it('renders with yellow warning level', () => {
    render(<UsageBar ratio={0.5} warningLevel="yellow" />);
    const bar = document.querySelector('.bg-\\[\\#F59E0B\\]');
    expect(bar).toBeInTheDocument();
  });

  it('renders with red warning level', () => {
    render(<UsageBar ratio={0.5} warningLevel="red" />);
    const bar = document.querySelector('.bg-\\[\\#EF4444\\]');
    expect(bar).toBeInTheDocument();
  });

  it('shows percentage label when showLabel is true', () => {
    render(<UsageBar ratio={0.55} warningLevel="none" showLabel />);
    expect(screen.getByText('55%')).toBeInTheDocument();
  });

  it('clips ratio to 0-100 range', () => {
    render(<UsageBar ratio={1.5} warningLevel="none" showLabel />);
    expect(screen.getByText('100%')).toBeInTheDocument();
  });

  it('handles zero ratio', () => {
    render(<UsageBar ratio={0} warningLevel="none" showLabel />);
    expect(screen.getByText('0%')).toBeInTheDocument();
  });
});
